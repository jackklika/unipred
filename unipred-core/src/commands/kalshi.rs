use super::Command;
use crate::UnipredCore;
use crate::clients::kalshi::Market;
use async_trait::async_trait;
use anyhow::Result;

pub struct FetchKalshiMarkets {
    pub limit: Option<i64>,
    pub cursor: Option<String>,
    pub event_ticker: Option<String>,
    pub series_ticker: Option<String>,
    pub max_close_ts: Option<i64>,
    pub min_close_ts: Option<i64>,
    pub status: Option<String>,
    pub tickers: Option<String>,
}

impl FetchKalshiMarkets {
    pub fn new() -> Self {
        Self {
            limit: None,
            cursor: None,
            event_ticker: None,
            series_ticker: None,
            max_close_ts: None,
            min_close_ts: None,
            status: None,
            tickers: None,
        }
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

    pub fn with_max_close_ts(mut self, ts: i64) -> Self {
        self.max_close_ts = Some(ts);
        self
    }
    
    pub fn with_min_close_ts(mut self, ts: i64) -> Self {
        self.min_close_ts = Some(ts);
        self
    }
}

#[async_trait]
impl Command for FetchKalshiMarkets {
    type Response = (Option<String>, Vec<Market>);

    async fn execute(&self, core: &UnipredCore) -> Result<Self::Response> {
        let result = core.kalshi.get_multiple_markets(
            self.limit,
            self.cursor.clone(),
            self.event_ticker.clone(),
            self.series_ticker.clone(),
            self.max_close_ts,
            self.min_close_ts,
            self.status.clone(),
            self.tickers.clone(),
        ).await?;

        Ok(result)
    }
}