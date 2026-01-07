pub mod clients;
pub mod commands;
pub mod domain;
pub mod storage;
pub mod ml;
pub mod ingestion;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/unipred.rs"));
}

use clients::kalshi::{Kalshi, TradingEnvironment};
use clients::polymarket::ClobClient;
use commands::markets::FetchMarkets;
use commands::quote::GetMarketQuote;
use commands::Command;
use domain::MarketSource;
use proto::{FetchedMarketList, MarketQuote};

pub struct UnipredCore {
    pub kalshi: Kalshi,
    pub polymarket: ClobClient,
}

impl UnipredCore {
    pub fn new(_config: String) -> Self {
        // In a real app, parse config to set up auth/environments
        Self {
            kalshi: Kalshi::new(TradingEnvironment::LiveMarketMode),
            polymarket: ClobClient::new("https://clob.polymarket.com"),
        }
    }

    /// High-level API to fetch markets
    pub async fn fetch_markets(
        &self,
        exchange: Option<MarketSource>,
        limit: Option<i64>,
        cursor: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<FetchedMarketList> {
        let mut cmd = FetchMarkets::new().with_exchange(exchange);

        if let Some(l) = limit {
            cmd = cmd.with_limit(l);
        } else {
            cmd = cmd.with_limit(100);
        }

        if let Some(c) = cursor {
            cmd = cmd.with_cursor(c);
        }

        if let Some(s) = status {
            cmd = cmd.with_status(s);
        }

        cmd.execute(self).await
    }

    /// High-level API to get a quote
    pub async fn get_quote(
        &self,
        ticker: String,
        exchange: Option<MarketSource>,
    ) -> anyhow::Result<MarketQuote> {
        let cmd = GetMarketQuote::new(ticker, exchange);
        cmd.execute(self).await
    }
}
