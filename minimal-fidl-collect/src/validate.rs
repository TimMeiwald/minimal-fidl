//! Whole-file checking after mutation. See `DESIGN.md` §7.
//!
//! Parsing is strict: a `.fidl` with duplicate names fails to build a tree at all.
//! This module exists for the other direction — a program that has been editing
//! the tree and wants to know whether the result is still sound, without having
//! to keep every intermediate state legal.

use std::fmt;

use crate::{node_ref::NodeRef, FidlFile, NodePath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub path: NodePath,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{label}: {} at {}", self.message, self.path)
    }
}

impl FidlFile {
    /// Check the whole tree and report what is wrong, rather than stopping at the
    /// first problem.
    ///
    /// Returns an empty vector for a sound file. Bulk edits can therefore pass
    /// through illegal intermediate states and be checked once at the end.
    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for node in self.nodes() {
            self.check_duplicate_children(&node, &mut diagnostics);
            self.check_names(&node, &mut diagnostics);
            self.check_version(&node, &mut diagnostics);
        }
        diagnostics
    }

    /// Two siblings of the same kind sharing a name.
    fn check_duplicate_children(&self, node: &NodeRef<'_>, out: &mut Vec<Diagnostic>) {
        let children = node.children();
        for (i, a) in children.iter().enumerate() {
            let (Some(a_name), a_kind) = (a.name(), a.kind_name()) else {
                continue;
            };
            for b in &children[i + 1..] {
                if b.kind_name() == a_kind && b.name() == Some(a_name) {
                    out.push(Diagnostic {
                        severity: Severity::Error,
                        path: self.path_of(b.id()).unwrap_or_default(),
                        message: format!("duplicate {a_kind} '{a_name}'"),
                    });
                }
            }
        }
    }

    /// Names that exist but are empty. The parser rejects these, so they can only
    /// appear via construction or mutation.
    fn check_names(&self, node: &NodeRef<'_>, out: &mut Vec<Diagnostic>) {
        if let NodeRef::TypeCollection(tc) = node {
            // The grammar allows this; it is only a problem because nothing can
            // reference the collection by name.
            if tc.is_anonymous() {
                out.push(Diagnostic {
                    severity: Severity::Warning,
                    path: self.path_of(node.id()).unwrap_or_default(),
                    message: "type collection has no name, so nothing can refer to it".to_string(),
                });
            }
            return;
        }
        if let Some(name) = node.name() {
            if name.trim().is_empty() {
                out.push(Diagnostic {
                    severity: Severity::Error,
                    path: self.path_of(node.id()).unwrap_or_default(),
                    message: format!("{} has an empty name", node.kind_name()),
                });
            }
        }
    }

    /// A version is only meaningful with both parts present.
    fn check_version(&self, node: &NodeRef<'_>, out: &mut Vec<Diagnostic>) {
        let NodeRef::Version(version) = node else {
            return;
        };
        if version.major.is_none() || version.minor.is_none() {
            out.push(Diagnostic {
                severity: Severity::Warning,
                path: self.path_of(node.id()).unwrap_or_default(),
                message: "version is missing a major or minor part".to_string(),
            });
        }
    }
}

// NOTE: unresolved type references are deliberately *not* checked here. Doing it
// properly needs cross-file import resolution, which is an explicit non-goal
// (`DESIGN.md` §2); without it, every file that imports a type would report false
// errors. It belongs with the import-following work in §11.
