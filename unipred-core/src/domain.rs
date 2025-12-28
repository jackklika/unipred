use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketQuote {
    pub ticker: String,
    pub price: Decimal,
    pub volume: Decimal,
    pub source: MarketSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketSource {
    Kalshi,
    Polymarket,
    Unknown,
}
