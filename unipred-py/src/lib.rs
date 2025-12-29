use pyo3::prelude::*;
use unipred_core::UnipredCore as CoreUnipred;
use unipred_core::commands::quote::GetMarketQuote;
use unipred_core::commands::kalshi::FetchKalshiMarkets;
use unipred_core::commands::Command;

#[pyclass]
struct UnipredCore {
    inner: CoreUnipred,
    rt: tokio::runtime::Runtime,
}

#[pymethods]
impl UnipredCore {
    #[new]
    fn new(config: String) -> Self {
        UnipredCore {
            inner: CoreUnipred::new(config),
            rt: tokio::runtime::Runtime::new().unwrap(),
        }
    }

    fn login(&mut self, email: String, password: String) -> PyResult<()> {
         let result = self.rt.block_on(async {
            self.inner.kalshi.login(&email, &password).await
        });

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn login_apikey(&mut self, key_id: String, private_key_path: String) -> PyResult<()> {
        let result = self.rt.block_on(async {
            self.inner.kalshi.login_apikey_from_path(&key_id, &private_key_path).await
        });

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature = (limit=100, cursor=None, status=None, min_close_ts=None, max_close_ts=None))]
    fn fetch_kalshi_markets(
        &self, 
        limit: i64, 
        cursor: Option<String>,
        status: Option<String>,
        min_close_ts: Option<i64>,
        max_close_ts: Option<i64>
    ) -> PyResult<String> {
        let mut cmd = FetchKalshiMarkets::new()
            .with_limit(limit);
        
        if let Some(c) = cursor {
            cmd = cmd.with_cursor(c);
        }
        if let Some(s) = status {
            cmd = cmd.with_status(s);
        }
        if let Some(ts) = min_close_ts {
            cmd = cmd.with_min_close_ts(ts);
        }
        if let Some(ts) = max_close_ts {
            cmd = cmd.with_max_close_ts(ts);
        }

        let result = self.rt.block_on(async {
            cmd.execute(&self.inner).await
        });

        match result {
            Ok((next_cursor, markets)) => {
                let response = serde_json::json!({
                    "cursor": next_cursor,
                    "markets": markets
                });
                Ok(response.to_string())
            },
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature = (ticker, exchange=None))]
    fn get_quote(&self, ticker: String, exchange: Option<String>) -> PyResult<String> {
        let source = match exchange.as_deref() {
            Some("kalshi") => Some(unipred_core::domain::MarketSource::Kalshi),
            Some("polymarket") => Some(unipred_core::domain::MarketSource::Polymarket),
            Some(s) => return Err(pyo3::exceptions::PyValueError::new_err(format!("Unknown exchange: {}", s))),
            None => None,
        };

        let cmd = GetMarketQuote::new(ticker, source);

        // Block until the future completes
        let result = self.rt.block_on(async {
            cmd.execute(&self.inner).await
        });

        match result {
            Ok(quote) => Ok(format!("{:?}", quote)),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn unipred_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<UnipredCore>()?;
    Ok(())
}