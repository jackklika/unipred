pub mod clients;
pub mod commands;
pub mod domain;

use clients::kalshi::{Kalshi, TradingEnvironment};
use clients::polymarket::ClobClient;

pub struct UnipredCore {
    pub kalshi: Kalshi,
    pub polymarket: ClobClient,
}

impl UnipredCore {
    pub fn new(_config: String) -> Self {
        // In a real app, parse config to set up auth/environments
        Self {
            kalshi: Kalshi::new(TradingEnvironment::DemoMode),
            polymarket: ClobClient::new("https://clob.polymarket.com"),
        }
    }
}
