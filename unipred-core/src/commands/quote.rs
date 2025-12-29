use super::Command;
use crate::UnipredCore;
use crate::domain::{MarketQuote, MarketSource};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use anyhow::Result;

pub struct GetMarketQuote {
    pub ticker: String,
    pub exchange: Option<MarketSource>,
}

impl GetMarketQuote {
    pub fn new(ticker: String, exchange: Option<MarketSource>) -> Self {
        Self { ticker, exchange }
    }
}

#[async_trait]
impl Command for GetMarketQuote {
    type Response = MarketQuote;

    async fn execute(&self, core: &UnipredCore) -> Result<Self::Response> {
        let source = self.exchange.clone().unwrap_or_else(|| MarketSource::detect(&self.ticker));

        match source {
            MarketSource::Kalshi => {
                let market = core.kalshi.get_single_market(&self.ticker).await?;
                
                Ok(MarketQuote {
                    ticker: market.ticker,
                    price: Decimal::from_i64(market.last_price).unwrap_or_default() / Decimal::new(100, 0),
                    volume: Decimal::from_i64(market.volume).unwrap_or_default(),
                    source: MarketSource::Kalshi,
                })
            },
            MarketSource::Polymarket => {
                let mid_resp = core.polymarket.get_midpoint(&self.ticker).await?;
                
                Ok(MarketQuote {
                    ticker: self.ticker.clone(),
                    price: mid_resp.mid,
                    volume: Decimal::new(0, 0),
                    source: MarketSource::Polymarket,
                })
            },
            MarketSource::Unknown => {
                anyhow::bail!("Could not determine exchange for ticker: {}", self.ticker);
            }
        }
    }
}