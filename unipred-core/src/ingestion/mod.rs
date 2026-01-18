use anyhow::{Context, Result};
use futures::future::{join_all, BoxFuture};
use crate::ml::Embedder;
use crate::storage::duck::DuckStore;
use crate::storage::lance::{LanceStore, MarketEmbedding, EventEmbedding};
use crate::UnipredCore;
use crate::domain::MarketSource;
use crate::commands::markets::FetchMarkets;
use crate::commands::events::FetchEvents;
use crate::commands::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

pub struct IngestionFilter {
    pub exchanges: Vec<MarketSource>,
    pub statuses: Vec<String>,
    pub max_pages: Option<usize>,
}

impl Default for IngestionFilter {
    fn default() -> Self {
        Self {
            exchanges: vec![],
            statuses: vec![],
            max_pages: None,
        }
    }
}

pub struct IngestionEngine {
    duck_store: Arc<Mutex<DuckStore>>,
    lance_store: Arc<LanceStore>,
    embedder: Arc<Embedder>,
}

impl IngestionEngine {
    pub async fn new(duck_path: &str, lance_path: &str) -> Result<Self> {
        // DuckStore::open is synchronous, run it? It's fine for init.
        let duck_store = DuckStore::open(duck_path).context("Failed to open DuckDB")?;
        let lance_store = LanceStore::connect(lance_path).await.context("Failed to connect to LanceDB")?;
        let embedder = Embedder::new().context("Failed to initialize Embedder")?;

        Ok(Self {
            duck_store: Arc::new(Mutex::new(duck_store)),
            lance_store: Arc::new(lance_store),
            embedder: Arc::new(embedder),
        })
    }

    pub async fn run<F>(&self, core: &UnipredCore, filters: IngestionFilter, cancel_check: Option<F>) -> Result<()>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        let exchanges = if filters.exchanges.is_empty() {
            vec![MarketSource::Kalshi, MarketSource::Polymarket]
        } else {
            filters.exchanges
        };

        let statuses = if filters.statuses.is_empty() {
             vec!["active".to_string(), "closed".to_string()]
        } else {
            filters.statuses
        };

        let mut tasks: Vec<BoxFuture<'_, Result<()>>> = Vec::new();

        for exchange in exchanges {
            // Polymarket API fetches all markets typically, filtering by status might be redundant or not supported 
            // in the same way as Kalshi via the unified command yet.
            if matches!(exchange, MarketSource::Polymarket) {
                // Just run once
                tasks.push(Box::pin(self.ingest_loop(core, exchange, None, &cancel_check, filters.max_pages)));
            } else {
                for status in &statuses {
                    tasks.push(Box::pin(self.ingest_loop(core, exchange.clone(), Some(status.clone()), &cancel_check, filters.max_pages)));
                    
                    if matches!(exchange, MarketSource::Kalshi) {
                        tasks.push(Box::pin(self.ingest_events_loop(core, exchange.clone(), Some(status.clone()), &cancel_check, filters.max_pages)));
                    }
                }
            }
        }
        
        let results = join_all(tasks).await;
        for res in results {
            res?;
        }
        
        println!("Creating vector index...");
        self.lance_store.create_index().await?;
        self.lance_store.create_events_index().await?;
        println!("Ingestion complete.");
        
        Ok(())
    }

    async fn ingest_events_loop<F>(
        &self,
        core: &UnipredCore,
        exchange: MarketSource,
        status: Option<String>,
        cancel_check: &Option<F>,
        max_pages: Option<usize>,
    ) -> Result<()>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        println!("Ingesting Events {:?} (Status: {:?})", exchange, status);
        let mut cursor: Option<String> = None;
        let mut page_count = 0;
        let mut total_events = 0;

        loop {
            if let Some(limit) = max_pages {
                if page_count >= limit {
                    println!("Reached max pages limit ({}) for Events {:?} {:?}", limit, exchange, status);
                    break;
                }
            }

            if let Some(check) = cancel_check {
                check()?;
            }

            let mut retries = 0;
            let max_retries = 5;
            
            let response = loop {
                let mut cmd = FetchEvents::new()
                    .with_exchange(Some(exchange.clone()))
                    .with_limit(100);
                
                if let Some(c) = cursor.clone() {
                    cmd = cmd.with_cursor(c);
                }
                if let Some(s) = status.clone() {
                    cmd = cmd.with_status(s);
                }

                match cmd.execute(core).await {
                    Ok(res) => break res,
                    Err(e) => {
                         if retries >= max_retries {
                            return Err(e).context("Max retries exceeded fetching events");
                        }
                        eprintln!("Error fetching events: {}. Retrying...", e);
                        sleep(Duration::from_secs(2u64.pow(retries))).await;
                        retries += 1;
                    }
                }
            };

            if response.events.is_empty() {
                break;
            }

            let batch_size = response.events.len();
            total_events += batch_size;
            println!("  Events Page {}: {} events", page_count, batch_size);

            {
                let mut duck = self.duck_store.lock().await;
                duck.insert_events_batch(&response.events)?;
            }

            let texts: Vec<String> = response.events.iter().map(|e| {
                format!(
                    "Title: {}\nSubtitle: {}",
                    e.title,
                    e.description
                )
            }).collect();

            let embedder = self.embedder.clone();
            let vectors = tokio::task::spawn_blocking(move || {
                embedder.embed_batch(texts)
            }).await??;

            let mut lance_records = Vec::with_capacity(batch_size);
            for (event, vector) in response.events.iter().zip(vectors.into_iter()) {
                 lance_records.push(EventEmbedding {
                    id: format!("{:?}:{}", exchange, event.ticker),
                    vector,
                    ticker: event.ticker.clone(),
                    source: event.source.clone(),
                    title: event.title.clone(),
                    description: event.description.clone(),
                    start_date: event.start_date.clone(),
                    end_date: event.end_date.clone(),
                });
            }

            self.lance_store.add_events(lance_records).await?;

            if response.cursor.is_empty() || Some(&response.cursor) == cursor.as_ref() {
                break;
            }
            cursor = Some(response.cursor);
            page_count += 1;
            sleep(Duration::from_millis(100)).await;
        }

        println!("Finished Events {:?}: {} total events.", exchange, total_events);
        Ok(())
    }

    async fn ingest_loop<F>(
        &self,
        core: &UnipredCore,
        exchange: MarketSource,
        status: Option<String>,
        cancel_check: &Option<F>,
        max_pages: Option<usize>,
    ) -> Result<()>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        println!("Ingesting {:?} (Status: {:?})", exchange, status);
        let mut cursor: Option<String> = None;
        let mut page_count = 0;
        let mut total_markets = 0;

        loop {
            if let Some(limit) = max_pages {
                if page_count >= limit {
                    println!("Reached max pages limit ({}) for {:?} {:?}", limit, exchange, status);
                    break;
                }
            }

            if let Some(check) = cancel_check {
                check()?;
            }

            let mut retries = 0;
            let max_retries = 5;
            let response = loop {
                let mut cmd = FetchMarkets::new()
                    .with_exchange(Some(exchange.clone()))
                    .with_limit(100);
                
                if let Some(c) = cursor.clone() {
                    cmd = cmd.with_cursor(c);
                }
                if let Some(s) = status.clone() {
                    // Map unified 'active' status to Kalshi API 'open'
                    let api_status = if matches!(exchange, MarketSource::Kalshi) && s == "active" {
                        "open".to_string()
                    } else {
                        s
                    };
                    cmd = cmd.with_status(api_status);
                }

                match cmd.execute(core).await {
                    Ok(res) => break res,
                    Err(e) => {
                        if retries >= max_retries {
                            return Err(e).context("Max retries exceeded");
                        }
                        eprintln!("Error fetching markets: {}. Retrying...", e);
                        sleep(Duration::from_secs(2u64.pow(retries))).await;
                        retries += 1;
                    }
                }
            };

            if response.markets.is_empty() {
                break;
            }

            let batch_size = response.markets.len();
            total_markets += batch_size;
            println!("  Page {}: {} markets", page_count, batch_size);

            // 1. DuckDB Store
            {
                // DuckDB operations are synchronous and fast for batch inserts
                let mut duck = self.duck_store.lock().await;
                duck.insert_batch(&response.markets)?;
            }

            // 2. Embeddings
            let texts: Vec<String> = response.markets.iter().map(|m| {
                format!(
                    "Title: {}\nDescription: {}\nOutcomes: {}",
                    m.title,
                    m.description,
                    m.outcomes.join(", ")
                )
            }).collect();

            let embedder = self.embedder.clone();
            let vectors = tokio::task::spawn_blocking(move || {
                embedder.embed_batch(texts)
            }).await??;

            // 3. LanceDB Store
            let mut lance_records = Vec::with_capacity(batch_size);
            for (market, vector) in response.markets.iter().zip(vectors.into_iter()) {
                lance_records.push(MarketEmbedding {
                    id: format!("{:?}:{}", exchange, market.ticker),
                    vector,
                    ticker: market.ticker.clone(),
                    source: market.source.clone(),
                    title: market.title.clone(),
                    description: market.description.clone(),
                    outcomes: market.outcomes.join(", "),
                });
            }
            
            self.lance_store.add_markets(lance_records).await?;

            // Pagination logic
            if response.cursor.is_empty() || Some(&response.cursor) == cursor.as_ref() {
                break;
            }
            cursor = Some(response.cursor);
            page_count += 1;
            
            // Nice to API
            sleep(Duration::from_millis(100)).await;
        }
        
        println!("Finished {:?}: {} total markets.", exchange, total_markets);
        Ok(())
    }
}