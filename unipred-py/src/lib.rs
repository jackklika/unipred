use pyo3::prelude::*;
use unipred_core::UnipredCore as CoreUnipred;
use unipred_core::commands::quote::GetMarketQuote;
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

    fn get_quote(&self, ticker: String) -> PyResult<String> {
        let cmd = GetMarketQuote::new(ticker);

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
