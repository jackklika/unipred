use prost::Message;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use unipred_core::UnipredCore as CoreUnipred;
use unipred_core::commands::quote::GetMarketQuote;
use unipred_core::commands::markets::FetchMarkets;
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

    #[pyo3(signature = (exchange=None, limit=100, cursor=None, status=None))]
    fn _fetch_markets_bytes(
        &self,
        exchange: Option<String>,
        limit: i64,
        cursor: Option<String>,
        status: Option<String>,
    ) -> PyResult<Py<PyBytes>> {
        let source = match exchange.as_deref() {
            Some("kalshi") => Some(unipred_core::domain::MarketSource::Kalshi),
            Some("polymarket") => Some(unipred_core::domain::MarketSource::Polymarket),
            Some(s) => return Err(pyo3::exceptions::PyValueError::new_err(format!("Unknown exchange: {}", s))),
            None => None,
        };

        let mut cmd = FetchMarkets::new()
            .with_limit(limit)
            .with_exchange(source);
        
        if let Some(c) = cursor {
            cmd = cmd.with_cursor(c);
        }
        if let Some(s) = status {
            cmd = cmd.with_status(s);
        }

        let result = self.rt.block_on(async {
            cmd.execute(&self.inner).await
        });

        match result {
            Ok(fetched_market_list) => {
                let mut buf = Vec::new();
                fetched_market_list
                    .encode(&mut buf)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                Python::with_gil(|py| Ok(PyBytes::new(py, &buf).into()))
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    #[pyo3(signature = (ticker, exchange=None))]
    fn _get_quote_bytes(&self, ticker: String, exchange: Option<String>) -> PyResult<Py<PyBytes>> {
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
            Ok(quote) => {
                let mut buf = Vec::new();
                quote.encode(&mut buf).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
Python::with_gil(|py| Ok(PyBytes::new(py, &buf).into()))
            },
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
