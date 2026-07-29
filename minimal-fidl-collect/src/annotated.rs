//! Uniform annotation access. See `DESIGN.md` §5.
//!
//! Unlike the abandoned `Ordered` trait, this one is dyn-compatible and earns its
//! place: "walk the tree and rewrite every `@description`" becomes one pass.

use crate::{
    method::ParamList, Annotation, Attribute, EnumValue, Enumeration, Interface, Method, Structure,
    TypeCollection, TypeDef, VariableDeclaration,
};

pub trait Annotated {
    fn annotations(&self) -> &[Annotation];
    fn annotations_mut(&mut self) -> &mut Vec<Annotation>;

    fn annotation(&self, name: &str) -> Option<&Annotation> {
        self.annotations().iter().find(|a| a.name == name)
    }

    fn has_annotation(&self, name: &str) -> bool {
        self.annotation(name).is_some()
    }

    fn annotation_mut(&mut self, name: &str) -> Option<&mut Annotation> {
        self.annotations_mut().iter_mut().find(|a| a.name == name)
    }

    /// Replace the contents of an existing annotation, or append a new one.
    fn set_annotation(&mut self, name: &str, contents: impl Into<String>) {
        let contents = contents.into();
        match self.annotation_mut(name) {
            Some(existing) => existing.contents = contents,
            None => self
                .annotations_mut()
                .push(Annotation::create(name, contents)),
        }
    }

    fn remove_annotation(&mut self, name: &str) -> Option<Annotation> {
        let annotations = self.annotations_mut();
        let index = annotations.iter().position(|a| a.name == name)?;
        Some(annotations.remove(index))
    }
}

macro_rules! impl_annotated {
    ($($t:ty),+ $(,)?) => {
        $(
            impl Annotated for $t {
                fn annotations(&self) -> &[Annotation] { &self.annotations }
                fn annotations_mut(&mut self) -> &mut Vec<Annotation> { &mut self.annotations }
            }
        )+
    };
}

impl_annotated!(
    Interface,
    TypeCollection,
    Method,
    ParamList,
    Attribute,
    Structure,
    Enumeration,
    EnumValue,
    TypeDef,
    VariableDeclaration,
);
