use pyo3::prelude::*;

/// This enum represents whether a diffed result
/// has a significant, less significant, minor or no change
#[pyclass(name = "FidlDiff", frozen)]
#[derive(Clone, Debug)]
pub enum FidlDiff {
    MAJOR,
    MINOR,
    PATCH,
    IDENTICAL,
}
