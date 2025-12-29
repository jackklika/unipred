use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketSource {
    Kalshi,
    Polymarket,
    Unknown,
}

impl MarketSource {
    pub fn detect(ticker: &str) -> Self {
        if ticker.starts_with("KX") {
            MarketSource::Kalshi
        } else if ticker.starts_with("0x") {
            MarketSource::Polymarket
        } else {
            MarketSource::Unknown
        }
    }
}