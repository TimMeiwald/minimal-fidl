use std::ops::Deref;

use pyo3::prelude::*;

/// This enum represents whether a diffed result
/// has a significant, less significant, minor or no change
#[pyclass(name = "FidlDiff", frozen)]
#[derive(Clone, Debug)]
pub enum FidlDiff {
    MAJOR = 1,
    MINOR = 2,
    PATCH = 3,
    IDENTICAL = 4,
}
#[pymethods]
impl FidlDiff{
    fn __int__(&self) -> i32{
        match self{
            Self::MAJOR => 1,
            Self::MINOR => 2,
            Self::PATCH => 3,
            Self::IDENTICAL => 4,
        }
    }

    fn __eq__(&self, other: &Self) -> bool{
        self.__int__() == other.__int__()
    }
}