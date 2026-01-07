use anyhow::{Context, Result};
use crate::ml::Embedder;
use crate::storage::duck::DuckStore;
use crate::storage::lance::{LanceStore, MarketEmbedding};
use crate::UnipredCore;
use crate::domain::MarketSource;
use crate::commands::markets::FetchMarkets;
use crate::commands::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

pub struct IngestionFilter {
    pub exchanges: Vec<MarketSource>,
    pub statuses: Vec<String>,
}

impl Default for IngestionFilter {
    fn default() -> Self {
        Self {
            exchanges: vec![],
            statuses: vec![],
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

    pub async fn run(&self, core: &UnipredCore, filters: IngestionFilter) -> Result<()> {
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

        for exchange in exchanges {
            // Polymarket API fetches all markets typically, filtering by status might be redundant or not supported 
            // in the same way as Kalshi via the unified command yet.
            if matches!(exchange, MarketSource::Polymarket) {
                // Just run once
                self.ingest_loop(core, exchange, None).await?;
            } else {
                for status in &statuses {
                    self.ingest_loop(core, exchange.clone(), Some(status.clone())).await?;
                }
            }
        }
        
        println!("Creating vector index...");
        self.lance_store.create_index().await?;
        println!("Ingestion complete.");
        
        Ok(())
    }

    async fn ingest_loop(&self, core: &UnipredCore, exchange: MarketSource, status: Option<String>) -> Result<()> {
        println!("Ingesting {:?} (Status: {:?})", exchange, status);
        let mut cursor: Option<String> = None;
        let mut page_count = 0;
        let mut total_markets = 0;

        loop {
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

            let vectors = self.embedder.embed_batch(texts)?;

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