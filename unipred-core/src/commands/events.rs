use super::Command;
use crate::UnipredCore;

use crate::domain::MarketSource;
use async_trait::async_trait;
use anyhow::Result;
use crate::proto::{FetchedEvent, FetchedEventList};

// TODO: Consider migrating this and FetchMarkets to a Repository pattern eventually.
// Currently, these commands act as the adapter layer between specific exchange clients
// and the unified domain model.

pub struct FetchEvents {
    pub exchange: Option<MarketSource>,
    pub limit: Option<i64>,
    pub cursor: Option<String>,
    pub status: Option<String>,
}

impl FetchEvents {
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
impl Command for FetchEvents {
    type Response = FetchedEventList;

    async fn execute(&self, core: &UnipredCore) -> Result<Self::Response> {
        let source = self.exchange.unwrap_or(MarketSource::Kalshi);

        match source {
            MarketSource::Kalshi => {
                let api_status = if let Some(s) = &self.status {
                     if s == "active" { Some("open".to_string()) } else { Some(s.clone()) }
                } else {
                    None
                };

                let (next_cursor, events) = core
                    .kalshi
                    .get_multiple_events(
                        self.limit,
                        self.cursor.clone(),
                        api_status,
                        None,
                        None
                    )
                    .await?;

                let unified_events = events
                    .into_iter()
                    .map(|e| FetchedEvent {
                        ticker: e.event_ticker.clone(),
                        title: e.title,
                        source: "Kalshi".to_string(),
                        description: e.sub_title,
                        start_date: e.strike_date.unwrap_or_default(),
                        end_date: "".to_string(),
                        url: format!("https://kalshi.com/events/{}", e.event_ticker),
                    })
                    .collect();

                Ok(FetchedEventList {
                    cursor: next_cursor.unwrap_or_default(),
                    events: unified_events,
                })
            }
            MarketSource::Polymarket => {
                // Polymarket events fetching not yet implemented in this command
                // Their "markets" API returns markets which are grouped by event, 
                // but dedicated event fetching might require a different endpoint or aggregation.
                Ok(FetchedEventList {
                    cursor: "".to_string(),
                    events: vec![],
                })
            }
            MarketSource::Unknown => {
                anyhow::bail!("Cannot fetch events for unknown exchange");
            }
        }
    }
}