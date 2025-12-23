use pyo3::prelude::*;
use unipred_core::UnipredCore as CoreUnipred;

#[pyclass]
struct UnipredCore {
    inner: CoreUnipred,
}

#[pymethods]
impl UnipredCore {
    #[new]
    fn new(config: String) -> Self {
        UnipredCore {
            inner: CoreUnipred::new(config),
        }
    }

    fn execute(&self) -> String {
        self.inner.execute()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn unipred_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<UnipredCore>()?;
    Ok(())
}
