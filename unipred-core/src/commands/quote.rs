use super::Command;
use crate::UnipredCore;
use crate::domain::MarketSource;
use crate::proto::MarketQuote;
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use anyhow::Result;
use chrono::Utc;

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
                    source: "Kalshi".to_string(),
                    price: (Decimal::from_i64(market.last_price).unwrap_or_default() / Decimal::new(100, 0)).to_string(),
                    bid: (Decimal::from_i64(market.yes_bid).unwrap_or_default() / Decimal::new(100, 0)).to_string(),
                    ask: (Decimal::from_i64(market.yes_ask).unwrap_or_default() / Decimal::new(100, 0)).to_string(),
                    volume: (Decimal::from_i64(market.volume).unwrap_or_default()).to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                })
            },
            MarketSource::Polymarket => {
                let book = core.polymarket.get_order_book(&self.ticker).await?;

                let best_bid = book.bids.first().map(|o| o.price);
                let best_ask = book.asks.first().map(|o| o.price);

                let price = match (best_bid, best_ask) {
                    (Some(b), Some(a)) => (b + a) / Decimal::new(2, 0),
                    (Some(b), None) => b,
                    (None, Some(a)) => a,
                    (None, None) => Decimal::ZERO,
                };

                Ok(MarketQuote {
                    ticker: self.ticker.clone(),
                    source: "Polymarket".to_string(),
                    price: price.to_string(),
                    bid: best_bid.map(|v| v.to_string()).unwrap_or_default(),
                    ask: best_ask.map(|v| v.to_string()).unwrap_or_default(),
                    volume: "".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                })
            },
            MarketSource::Unknown => {
                anyhow::bail!("Could not determine exchange for ticker: {}", self.ticker);
            }
        }
    }
}