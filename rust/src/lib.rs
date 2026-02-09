use pyo3::prelude::*;

/// Mantle Rust core — PyO3 module for performance-critical PII operations.
#[pymodule]
fn mantle_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "0.1.0")?;
    Ok(())
}
