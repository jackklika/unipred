use super::Command;
use crate::UnipredCore;

use crate::domain::MarketSource;
use async_trait::async_trait;
use anyhow::Result;
use crate::proto::{FetchedMarket, FetchedMarketList};

pub struct FetchMarkets {
    pub exchange: Option<MarketSource>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
    pub status: Option<String>,
}

impl FetchMarkets {
    pub fn new() -> Self {
        Self {
            exchange: None,
            limit: None,
            cursor: None,
            status: None,
        }
    }

    pub fn with_exchange(mut self, exchange: Option<MarketSource>) -> Self {
        self.exchange = exchange;
        self
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_cursor(mut self, cursor: String) -> Self {
        self.cursor = Some(cursor);
        self
    }
    
    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }
}

#[async_trait]
impl Command for FetchMarkets {
    type Response = FetchedMarketList;

    async fn execute(&self, core: &UnipredCore) -> Result<Self::Response> {
        // Default to Kalshi if not specified, or support multi-fetch logic later
        let source = self.exchange.unwrap_or(MarketSource::Kalshi);

        match source {
            MarketSource::Kalshi => {
                let (next_cursor, markets) = core
                    .kalshi
                    .get_multiple_markets(
                        self.limit,
                        self.cursor.clone(),
                        None,
                        None,
                        None,
                        None,
                        self.status.clone(),
                        None,
                    )
                    .await?;

                let unified_markets = markets
                    .into_iter()
                    .filter(|m| m.mve_collection_ticker.is_none())
                    .map(|m| FetchedMarket {
                        ticker: m.ticker.clone(),
                        title: m.title,
                        source: "Kalshi".to_string(),
                        status: m.status,
                        description: m.subtitle,
                        outcomes: vec![m.yes_sub_title, m.no_sub_title],
                        start_date: m.open_time.clone(),
                        end_date: m.close_time.clone(),
                        volume: m.volume.to_string(),
                        liquidity: m.liquidity.to_string(),
                        url: format!("https://kalshi.com/markets/{}", m.ticker),
                    })
                    .collect();

                Ok(FetchedMarketList {
                    cursor: next_cursor.unwrap_or_default(),
                    markets: unified_markets,
                })
            }
            MarketSource::Polymarket => {
                // Simplified Polymarket fetching
                let markets = core
                    .polymarket
                    .get_markets(self.cursor.as_deref())
                    .await?;

                let unified_markets = markets
                    .data
                    .into_iter()
                    .map(|m| FetchedMarket {
                        ticker: m.tokens[0].token_id.clone(), // Using token_id as ticker for consistency with get_quote
                        title: m.question,
                        source: "Polymarket".to_string(),
                        status: if m.active {
                            "active".to_string()
                        } else {
                            "closed".to_string()
                        },
                        description: m.description,
                        outcomes: m.tokens.iter().map(|t| t.outcome.clone()).collect(),
                        start_date: m.game_start_time.unwrap_or_default(),
                        end_date: m.end_date_iso.unwrap_or_default(),
                        volume: "0".to_string(),
                        liquidity: "0".to_string(),
                        url: format!("https://polymarket.com/event/{}", m.market_slug),
                    })
                    .collect();

                Ok(FetchedMarketList {
                    cursor: markets.next_cursor.unwrap_or_default(),
                    markets: unified_markets,
                })
            }
            MarketSource::Unknown => {
                anyhow::bail!("Cannot fetch markets for unknown exchange");
            }
        }
    }
}