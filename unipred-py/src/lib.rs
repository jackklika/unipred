use prost::Message;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use unipred_core::UnipredCore as CoreUnipred;
use unipred_core::commands::quote::GetMarketQuote;
use unipred_core::commands::markets::FetchMarkets;
use unipred_core::commands::Command;
use unipred_core::storage::lance::{LanceStore, MarketEmbedding};
use unipred_core::ingestion::{IngestionEngine, IngestionFilter};

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

    #[pyo3(signature = (db_path, lancedb_path, exchanges=None, statuses=None))]
    fn ingest_all(
        &self,
        db_path: String,
        lancedb_path: String,
        exchanges: Option<Vec<String>>,
        statuses: Option<Vec<String>>,
    ) -> PyResult<()> {
        let mut filters = IngestionFilter::default();

        if let Some(exs) = exchanges {
            filters.exchanges = exs
                .into_iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "kalshi" => Some(unipred_core::domain::MarketSource::Kalshi),
                    "polymarket" => Some(unipred_core::domain::MarketSource::Polymarket),
                    _ => None,
                })
                .collect();
        }

        if let Some(st) = statuses {
            filters.statuses = st;
        }

        let result = self.rt.block_on(async {
            let engine = IngestionEngine::new(&db_path, &lancedb_path).await?;
            engine.run(&self.inner, filters).await
        });

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Ingestion error: {}",
                e
            ))),
        }
    }
}

#[pyclass]
struct PyLanceDb {
    inner: LanceStore,
    rt: tokio::runtime::Runtime,
}

#[pymethods]
impl PyLanceDb {
    #[staticmethod]
    fn connect(uri: String) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let inner = rt.block_on(async { LanceStore::connect(&uri).await })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(PyLanceDb { inner, rt })
    }

    fn add_markets(&self, markets: Vec<Bound<'_, PyAny>>) -> PyResult<()> {
        let mut embeddings = Vec::with_capacity(markets.len());
        for m in markets {
            let id: String = m.getattr("id")?.extract()?;
            let vector: Vec<f32> = m.getattr("vector")?.extract()?;
            let ticker: String = m.getattr("ticker")?.extract()?;
            let source: String = m.getattr("source")?.extract()?;
            let title: String = m.getattr("title")?.extract()?;
            let description: String = m.getattr("description")?.extract()?;
            let outcomes: String = m.getattr("outcomes")?.extract()?;

            embeddings.push(MarketEmbedding {
                id,
                vector,
                ticker,
                source,
                title,
                description,
                outcomes,
            });
        }

        self.rt
            .block_on(async { self.inner.add_markets(embeddings).await })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        Ok(())
    }

    fn create_index(&self) -> PyResult<()> {
        self.rt
            .block_on(async { self.inner.create_index().await })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> PyResult<Vec<(String, String, String, String, String, String)>> {
        let results = self.rt
            .block_on(async { self.inner.search(query_vector, limit).await })
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Return simplified tuple: (id, ticker, source, title, description, outcomes)
        let py_results = results
            .into_iter()
            .map(|m| {
                (
                    m.id,
                    m.ticker,
                    m.source,
                    m.title,
                    m.description,
                    m.outcomes,
                )
            })
            .collect();

        Ok(py_results)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn unipred_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<UnipredCore>()?;
    m.add_class::<PyLanceDb>()?;
    Ok(())
}
