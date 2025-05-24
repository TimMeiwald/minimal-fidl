use pyo3::prelude::*;
use rand::Rng;
use std::cmp::Ordering;
use std::io;


#[pyfunction]
fn respond_42() -> u8{
    42
}


/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn franca_idl_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(respond_42, m)?)?;

    Ok(())
}
