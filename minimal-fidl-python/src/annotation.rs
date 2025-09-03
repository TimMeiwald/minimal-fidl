use minimal_fidl_collect::Annotation;
use pyo3::prelude::*;

#[pyclass(name = "FidlAnnotation", frozen)]
#[derive(Clone, Debug)]
pub struct FidlAnnotation {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub contents: String,
}
#[pymethods]
impl FidlAnnotation {
    fn __str__(&self) -> String {
        format!("{:#?}", self)
    }
}
impl From<&Annotation> for FidlAnnotation {
    fn from(item: &Annotation) -> Self {
        FidlAnnotation {
            name: item.name.clone(),
            contents: item.contents.clone(),
        }
    }
}
