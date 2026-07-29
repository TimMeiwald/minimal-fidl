//! Addressing nodes by position in the tree. See `DESIGN.md` §5.
//!
//! Names are used where nodes have them and indices where they do not, so a path
//! stays readable in diff output and error messages.

use std::fmt;

use crate::{node::NodeId, node_ref::NodeRef, FidlFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    Named { kind: &'static str, name: String },
    Indexed { kind: &'static str, index: usize },
}

impl PathSegment {
    pub fn kind(&self) -> &'static str {
        match self {
            PathSegment::Named { kind, .. } | PathSegment::Indexed { kind, .. } => kind,
        }
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Named { kind, name } => write!(f, "{kind}({name})"),
            PathSegment::Indexed { kind, index } => write!(f, "{kind}[{index}]"),
        }
    }
}

/// The route from the file root to a node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodePath(pub Vec<PathSegment>);

impl NodePath {
    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, segment: PathSegment) {
        self.0.push(segment);
    }

    /// A path with `segment` appended, leaving `self` untouched.
    pub fn joined(&self, segment: PathSegment) -> Self {
        let mut next = self.clone();
        next.push(segment);
        next
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "<file>");
        }
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            write!(f, "{segment}")?;
        }
        Ok(())
    }
}

/// The segment naming a node among its siblings.
pub(crate) fn segment_for(node: &NodeRef<'_>, index: usize) -> PathSegment {
    match node.name() {
        Some(name) => PathSegment::Named {
            kind: node.kind_name(),
            name: name.to_string(),
        },
        None => PathSegment::Indexed {
            kind: node.kind_name(),
            index,
        },
    }
}

impl FidlFile {
    /// The path to the node with this id, or `None` if it is not in the tree.
    ///
    /// The file root has the empty path.
    pub fn path_of(&self, id: NodeId) -> Option<NodePath> {
        fn walk(node: NodeRef<'_>, target: NodeId, prefix: NodePath) -> Option<NodePath> {
            if node.id() == target {
                return Some(prefix);
            }
            for (index, child) in node.children().into_iter().enumerate() {
                let child_path = prefix.joined(segment_for(&child, index));
                if let Some(found) = walk(child, target, child_path) {
                    return Some(found);
                }
            }
            None
        }
        walk(self.as_node(), id, NodePath::default())
    }

    /// The node at this path, if one is there.
    pub fn at_path(&self, path: &NodePath) -> Option<NodeRef<'_>> {
        let mut node = self.as_node();
        for wanted in path.segments() {
            let children = node.children();
            let next = children
                .iter()
                .enumerate()
                .find(|(index, child)| &segment_for(child, *index) == wanted)?;
            node = *next.1;
        }
        Some(node)
    }
}
