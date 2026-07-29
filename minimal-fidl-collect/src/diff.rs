//! Structural comparison of two trees. See `DESIGN.md` §9.
//!
//! Nodes are matched by name where they have one and by position where they do
//! not, so renaming a method reads as a remove plus an add while reordering two
//! methods reads as a move.
//!
//! Because layout lives in the tree (`blank_lines_before`, comments), two
//! *unformatted* files can be compared directly — there is no need to normalise
//! them first.

use std::collections::BTreeMap;
use std::fmt;

use crate::{node_ref::NodeRef, path::segment_for, FidlFile, NodePath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffOptions {
    /// Ignore comments entirely: compare only what the interface means.
    pub ignore_comments: bool,
    /// Ignore blank-line grouping. On by default — spacing is rarely the point.
    pub ignore_layout: bool,
    /// Treat reordered siblings as unchanged.
    pub ignore_order: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            ignore_comments: false,
            ignore_layout: true,
            ignore_order: false,
        }
    }
}

impl DiffOptions {
    /// Compare meaning only: no comments, no layout, no ordering.
    pub fn semantic() -> Self {
        Self {
            ignore_comments: true,
            ignore_layout: true,
            ignore_order: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Added {
        path: NodePath,
        kind: &'static str,
        name: Option<String>,
    },
    Removed {
        path: NodePath,
        kind: &'static str,
        name: Option<String>,
    },
    Modified {
        path: NodePath,
        kind: &'static str,
        field: &'static str,
        before: String,
        after: String,
    },
    Moved {
        path: NodePath,
        kind: &'static str,
        from: usize,
        to: usize,
    },
}

impl Change {
    pub fn path(&self) -> &NodePath {
        match self {
            Change::Added { path, .. }
            | Change::Removed { path, .. }
            | Change::Modified { path, .. }
            | Change::Moved { path, .. } => path,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Change::Added { kind, .. }
            | Change::Removed { kind, .. }
            | Change::Modified { kind, .. }
            | Change::Moved { kind, .. } => kind,
        }
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Change::Added { path, kind, name } => {
                write!(f, "+ {kind} {} at {path}", name.as_deref().unwrap_or(""))
            }
            Change::Removed { path, kind, name } => {
                write!(f, "- {kind} {} at {path}", name.as_deref().unwrap_or(""))
            }
            Change::Modified {
                path,
                kind,
                field,
                before,
                after,
            } => write!(f, "~ {kind}.{field}: {before:?} -> {after:?} at {path}"),
            Change::Moved {
                path,
                kind,
                from,
                to,
            } => write!(f, "> {kind} moved {from} -> {to} at {path}"),
        }
    }
}

/// Compare two files.
///
/// The result is ordered by where each change was found, walking the tree from
/// the top down.
pub fn diff(before: &FidlFile, after: &FidlFile, options: &DiffOptions) -> Vec<Change> {
    let mut out = Vec::new();
    diff_nodes(
        before.as_node(),
        after.as_node(),
        &NodePath::default(),
        options,
        &mut out,
    );
    out
}

impl FidlFile {
    /// Convenience for [`diff`] with [`DiffOptions::default`].
    pub fn diff(&self, other: &FidlFile) -> Vec<Change> {
        diff(self, other, &DiffOptions::default())
    }
}

fn diff_nodes(
    before: NodeRef<'_>,
    after: NodeRef<'_>,
    path: &NodePath,
    options: &DiffOptions,
    out: &mut Vec<Change>,
) {
    for (field, b, a) in zip_fields(&before, &after) {
        if b != a {
            out.push(Change::Modified {
                path: path.clone(),
                kind: before.kind_name(),
                field,
                before: b,
                after: a,
            });
        }
    }

    if !options.ignore_layout {
        let b = before.meta().map(|m| m.blank_lines_before).unwrap_or(0);
        let a = after.meta().map(|m| m.blank_lines_before).unwrap_or(0);
        if b != a {
            out.push(Change::Modified {
                path: path.clone(),
                kind: before.kind_name(),
                field: "blank_lines_before",
                before: b.to_string(),
                after: a.to_string(),
            });
        }
    }

    diff_children(before, after, path, options, out);
}

/// Key a node is matched by among its siblings: its name where it has one, its
/// position among same-kind siblings where it does not.
///
/// The occurrence counter is applied to *every* key, not only anonymous ones.
/// A file may legally hold two interfaces of the same name (that is a
/// `validate()` diagnostic, not a parse error), and without it they collide in
/// the map and a file reports differences against itself.
fn child_keys<'a>(node: NodeRef<'a>, options: &DiffOptions) -> Vec<(String, usize, NodeRef<'a>)> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    node.children()
        .into_iter()
        .filter(|c| !(options.ignore_comments && matches!(c, NodeRef::Comment(_))))
        .enumerate()
        .map(|(index, child)| {
            let base = match child.name() {
                Some(name) => format!("{}:{}", child.kind_name(), name),
                None => child.kind_name().to_string(),
            };
            let occurrence = seen.entry(base.clone()).or_insert(0);
            let key = format!("{base}#{occurrence}");
            *occurrence += 1;
            (key, index, child)
        })
        .collect()
}

fn diff_children(
    before: NodeRef<'_>,
    after: NodeRef<'_>,
    path: &NodePath,
    options: &DiffOptions,
    out: &mut Vec<Change>,
) {
    let before_children = child_keys(before, options);
    let after_children = child_keys(after, options);

    let after_by_key: BTreeMap<&str, (usize, NodeRef<'_>)> = after_children
        .iter()
        .map(|(k, i, n)| (k.as_str(), (*i, *n)))
        .collect();
    let before_by_key: BTreeMap<&str, (usize, NodeRef<'_>)> = before_children
        .iter()
        .map(|(k, i, n)| (k.as_str(), (*i, *n)))
        .collect();

    for (key, before_index, before_child) in &before_children {
        let child_path = path.joined(segment_for(before_child, *before_index));
        match after_by_key.get(key.as_str()) {
            None => out.push(Change::Removed {
                path: child_path,
                kind: before_child.kind_name(),
                name: before_child.name().map(str::to_string),
            }),
            Some((after_index, after_child)) => {
                if !options.ignore_order && before_index != after_index {
                    out.push(Change::Moved {
                        path: child_path.clone(),
                        kind: before_child.kind_name(),
                        from: *before_index,
                        to: *after_index,
                    });
                }
                diff_nodes(*before_child, *after_child, &child_path, options, out);
            }
        }
    }

    for (key, after_index, after_child) in &after_children {
        if !before_by_key.contains_key(key.as_str()) {
            out.push(Change::Added {
                path: path.joined(segment_for(after_child, *after_index)),
                kind: after_child.kind_name(),
                name: after_child.name().map(str::to_string),
            });
        }
    }
}

/// The scalar fields of a node, paired between the two sides.
///
/// Kinds that differ have no comparable fields; that case cannot arise, since
/// the match key includes the kind.
fn zip_fields(before: &NodeRef<'_>, after: &NodeRef<'_>) -> Vec<(&'static str, String, String)> {
    let b = fields(before);
    let a = fields(after);
    b.into_iter()
        .zip(a)
        .map(|((name, bv), (_, av))| (name, bv, av))
        .collect()
}

fn fields(node: &NodeRef<'_>) -> Vec<(&'static str, String)> {
    match node {
        NodeRef::File(_) | NodeRef::ParamList(_) => Vec::new(),
        NodeRef::Package(p) => vec![("path", p.path.join("."))],
        NodeRef::ImportModel(i) => vec![("file_path", i.file_path.display().to_string())],
        NodeRef::ImportNamespace(n) => vec![
            ("import", n.import.join(".")),
            ("wildcard", n.wildcard.to_string()),
            ("from", n.from.display().to_string()),
        ],
        NodeRef::Interface(i) => vec![("name", i.name.clone())],
        NodeRef::TypeCollection(t) => vec![("name", t.name.clone())],
        NodeRef::Version(v) => vec![
            ("major", format_opt(v.major)),
            ("minor", format_opt(v.minor)),
        ],
        NodeRef::Method(m) => vec![("name", m.name.clone())],
        NodeRef::Attribute(a) => vec![("name", a.name.clone()), ("type", a.type_n.clone())],
        NodeRef::Structure(s) => vec![("name", s.name.clone())],
        NodeRef::Enumeration(e) => vec![("name", e.name.clone())],
        NodeRef::EnumValue(v) => vec![("name", v.name.clone()), ("value", format_opt(v.value))],
        NodeRef::TypeDef(t) => vec![
            ("name", t.name.clone()),
            ("type", t.type_n.clone()),
            ("is_array", t.is_array.to_string()),
        ],
        NodeRef::VariableDeclaration(v) => vec![
            ("name", v.name.clone()),
            ("type", v.type_n.clone()),
            ("is_array", v.is_array.to_string()),
        ],
        NodeRef::Annotation(a) => vec![
            ("name", a.name.clone()),
            ("contents", a.contents.trim().to_string()),
        ],
        NodeRef::Comment(c) => vec![("text", c.text.trim().to_string())],
    }
}

fn format_opt<T: fmt::Display>(value: Option<T>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "-".to_string(),
    }
}
