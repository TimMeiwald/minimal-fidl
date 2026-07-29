//! Core primitives shared by every AST node: identity, source spans, comments,
//! and layout-aware comparison.
//!
//! See `DESIGN.md` §4.3 and §6.

use minimal_fidl_parser::{BasicPublisher, Node, Rules};

/// A stable, per-file identifier for a node.
///
/// Ids are allocated from a counter owned by the [`crate::FidlFile`] the node
/// belongs to. They are:
///
/// - **stable** across sibling insertion, removal, and reordering;
/// - **never reused** within a single file's lifetime;
/// - **not** stable across a reparse — a fresh parse allocates fresh ids.
///
/// Nodes built by a builder start out [`NodeId::UNASSIGNED`] and are given a real
/// id when they are inserted into a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// The id of a node that has been constructed but not yet inserted into a file.
    pub const UNASSIGNED: NodeId = NodeId(0);

    pub fn is_assigned(self) -> bool {
        self != Self::UNASSIGNED
    }

    pub fn get(self) -> u32 {
        self.0
    }

    /// Rebuild an id from the number [`Self::get`] handed out.
    ///
    /// Ids are per-file counters, so one is only meaningful against the file it
    /// came from; anywhere else it simply fails to resolve. Exists for callers
    /// that have to round-trip an id through a numeric type — the Python binding
    /// exposes `node.id` as an `int`.
    pub fn from_raw(value: u32) -> Self {
        Self(value)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::UNASSIGNED
    }
}

/// Allocator for [`NodeId`]s. One lives on each [`crate::FidlFile`].
#[derive(Debug, Clone)]
pub struct NodeIdGen {
    next: u32,
}

impl NodeIdGen {
    pub fn new() -> Self {
        // 0 is reserved for NodeId::UNASSIGNED.
        Self { next: 1 }
    }

    /// The id that would be returned by the next call to [`Self::next_id`].
    pub fn peek(&self) -> NodeId {
        NodeId(self.next)
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("NodeId space exhausted (u32::MAX nodes in one file)");
        id
    }
}

impl Default for NodeIdGen {
    fn default() -> Self {
        Self::new()
    }
}

/// A byte range in the source a node was parsed from.
///
/// A span records where a node *originally* came from. It goes stale as soon as
/// the tree is mutated, and is only used for diagnostics and for reusing original
/// text when a subtree is known to be unchanged. Synthesised nodes have no span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "Span start must not exceed end");
        Self { start, end }
    }

    pub fn from_cst(node: &Node) -> Self {
        Self::new(node.start_position, node.end_position)
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// The source text this span covers.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start as usize..self.end as usize]
    }
}

/// Comparison that takes layout (spans, blank lines) into account.
///
/// The `PartialEq` impls on AST nodes deliberately ignore identity and layout, so
/// that a node which moved but did not change compares equal. When layout *is*
/// significant — the diff engine's `ignore_layout: false` mode — use this instead.
pub trait LayoutEq {
    fn eq_with_layout(&self, other: &Self) -> bool;
}

impl<T: LayoutEq> LayoutEq for Option<T> {
    fn eq_with_layout(&self, other: &Self) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(a), Some(b)) => a.eq_with_layout(b),
            _ => false,
        }
    }
}

impl<T: LayoutEq> LayoutEq for Vec<T> {
    fn eq_with_layout(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self
                .iter()
                .zip(other.iter())
                .all(|(a, b)| a.eq_with_layout(b))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    /// `// ...` — runs to the end of the line.
    Line,
    /// `/* ... */`
    Block,
}

/// A comment, preserved as a first-class node so that files round-trip.
///
/// `text` holds the *content* only; delimiters are stripped on construction and
/// re-added when printing.
#[derive(Debug, Clone)]
pub struct Comment {
    pub id: NodeId,
    pub span: Option<Span>,
    /// Blank lines that preceded this comment in the source. See `DESIGN.md` §4.3.
    pub blank_lines_before: u8,
    pub kind: CommentKind,
    pub text: String,
}

impl Comment {
    pub fn line(text: impl Into<String>) -> Self {
        Self {
            id: NodeId::UNASSIGNED,
            span: None,
            blank_lines_before: 0,
            kind: CommentKind::Line,
            text: text.into(),
        }
    }

    pub fn block(text: impl Into<String>) -> Self {
        Self {
            id: NodeId::UNASSIGNED,
            span: None,
            blank_lines_before: 0,
            kind: CommentKind::Block,
            text: text.into(),
        }
    }

    /// Build a comment from a `Rules::comment` or `Rules::multiline_comment` node.
    ///
    /// Returns `None` for any other rule, so callers can use it as a filter.
    pub fn from_cst(source: &str, node: &Node) -> Option<Self> {
        let raw = node.get_string(source);
        let (kind, text) = match node.rule {
            // `//` .. end of line. The newline is not part of the node.
            Rules::comment => (CommentKind::Line, raw.strip_prefix("//").unwrap_or(&raw)),
            // `/*` .. `*/`, both delimiters included in the node.
            Rules::multiline_comment => {
                let inner = raw.strip_prefix("/*").unwrap_or(&raw);
                (CommentKind::Block, inner.strip_suffix("*/").unwrap_or(inner))
            }
            _ => return None,
        };
        Some(Self {
            id: NodeId::UNASSIGNED,
            span: Some(Span::from_cst(node)),
            blank_lines_before: 0,
            kind,
            text: text.to_string(),
        })
    }

    /// The comment as it appears in source, delimiters included.
    pub fn to_source(&self) -> String {
        match self.kind {
            CommentKind::Line => format!("//{}", self.text),
            CommentKind::Block => format!("/*{}*/", self.text),
        }
    }
}

impl PartialEq for Comment {
    /// Ignores id, span, and blank lines — see [`LayoutEq`].
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.text == other.text
    }
}

impl Eq for Comment {}

impl LayoutEq for Comment {
    fn eq_with_layout(&self, other: &Self) -> bool {
        self == other && self.blank_lines_before == other.blank_lines_before
    }
}

/// Count blank lines in the source gap between two nodes.
///
/// `wsn` in the grammar consumes blank lines without emitting nodes, so this is
/// the only way to recover them. The gap between two adjacent members contains
/// nothing but whitespace, since comments are themselves nodes.
///
/// One newline means the next item is on the following line — zero blank lines.
pub(crate) fn count_blank_lines(source: &str, gap_start: u32, gap_end: u32) -> u8 {
    if gap_start >= gap_end || gap_end as usize > source.len() {
        return 0;
    }
    let newlines = source[gap_start as usize..gap_end as usize]
        .bytes()
        .filter(|b| *b == b'\n')
        .count();
    u8::try_from(newlines.saturating_sub(1)).unwrap_or(u8::MAX)
}

/// Fields every AST node carries: identity, provenance, and layout.
///
/// Embedded as a `meta` field rather than repeated on 15 structs.
#[derive(Debug, Clone, Default)]
pub struct NodeMeta {
    pub id: NodeId,
    pub span: Option<Span>,
    /// Blank lines between the previous sibling and this node's first comment
    /// (or the node itself, if it has no leading comments).
    pub blank_lines_before: u8,
    /// Comments immediately above this node, with no blank line between. They
    /// travel with the node: removing the node removes them too.
    pub leading_comments: Vec<Comment>,
    /// Comments inside the node's own header — between its annotation block and
    /// its opening brace. Rare, but they have nowhere else to live.
    pub header_comments: Vec<Comment>,
    /// Comments inside the node's span that follow its content, typically on the
    /// same line: `attribute UInt32 x // why`. Nodes with no member list of their
    /// own have nowhere else to put them, and dropping them loses source content.
    pub trailing_comments: Vec<Comment>,
    /// Set when the node may have been modified since it was parsed, which means
    /// its `span` no longer describes it and `Mode::Preserve` must re-format it
    /// rather than reuse the original text. See `DESIGN.md` §8.
    ///
    /// Deliberately conservative: handing out a `&mut` to a node marks it, whether
    /// or not the caller changes anything. Over-marking costs output fidelity;
    /// under-marking would emit stale text, which is a correctness bug.
    pub dirty: bool,
}

impl NodeMeta {
    pub fn from_cst(node: &Node) -> Self {
        Self {
            span: Some(Span::from_cst(node)),
            ..Default::default()
        }
    }
}

/// Common behaviour for every AST node.
pub trait AstNode {
    fn meta(&self) -> &NodeMeta;
    fn meta_mut(&mut self) -> &mut NodeMeta;

    fn id(&self) -> NodeId {
        self.meta().id
    }
    fn span(&self) -> Option<Span> {
        self.meta().span
    }
    fn blank_lines_before(&self) -> u8 {
        self.meta().blank_lines_before
    }
    fn trailing_comments(&self) -> &[Comment] {
        &self.meta().trailing_comments
    }
    fn leading_comments(&self) -> &[Comment] {
        &self.meta().leading_comments
    }
    fn leading_comments_mut(&mut self) -> &mut Vec<Comment> {
        self.mark_dirty();
        &mut self.meta_mut().leading_comments
    }

    fn is_dirty(&self) -> bool {
        self.meta().dirty
    }
    /// Mark this node as modified. Idempotent.
    fn mark_dirty(&mut self) {
        self.meta_mut().dirty = true;
    }
    /// Clear the modified flag, e.g. after re-parsing or writing out.
    fn mark_clean(&mut self) {
        self.meta_mut().dirty = false;
    }
}

/// Generates `impl AstNode` for a struct with a `meta: NodeMeta` field.
macro_rules! impl_ast_node {
    ($($t:ty),+ $(,)?) => {
        $(
            impl $crate::node::AstNode for $t {
                fn meta(&self) -> &$crate::node::NodeMeta { &self.meta }
                fn meta_mut(&mut self) -> &mut $crate::node::NodeMeta { &mut self.meta }
            }
        )+
    };
}
pub(crate) use impl_ast_node;

/// A container's ordered-member enum. Every such enum has a `Comment` variant so
/// that free-floating comments keep their position.
pub trait MemberEnum {
    fn from_comment(comment: Comment) -> Self;
}

/// Generates typed accessors over a container's ordered `members` field.
///
/// The accessors *filter* the ordered list rather than duplicating it, so they
/// cannot disagree with source order. See `DESIGN.md` §4.2.
#[macro_export]
macro_rules! member_accessors {
    ($enum:ident, $variant:ident, $ty:ty, $iter:ident, $iter_mut:ident, $get:ident, $get_mut:ident) => {
        pub fn $iter(&self) -> impl Iterator<Item = &$ty> {
            self.members.iter().filter_map(|m| match m {
                $enum::$variant(v) => Some(v),
                _ => None,
            })
        }

        /// Marks every yielded node dirty — see [`NodeMeta::dirty`].
        pub fn $iter_mut(&mut self) -> impl Iterator<Item = &mut $ty> {
            use $crate::node::AstNode as _;
            self.members.iter_mut().filter_map(|m| match m {
                $enum::$variant(v) => {
                    v.mark_dirty();
                    Some(v)
                }
                _ => None,
            })
        }

        pub fn $get(&self, name: &str) -> Option<&$ty> {
            self.$iter().find(|v| v.name == name)
        }

        /// Marks the returned node dirty — see [`NodeMeta::dirty`].
        pub fn $get_mut(&mut self, name: &str) -> Option<&mut $ty> {
            use $crate::node::AstNode as _;
            let found = self
                .members
                .iter_mut()
                .filter_map(|m| match m {
                    $enum::$variant(v) => Some(v),
                    _ => None,
                })
                .find(|v| v.name == name)?;
            found.mark_dirty();
            Some(found)
        }
    };
}

/// Generates typed add/remove operations over a container's ordered `members`.
///
/// Adding rejects a duplicate name, matching what parsing does. Removing a named
/// node takes its bound leading comments with it, since they are stored on the
/// node itself (`DESIGN.md` §4.3).
#[macro_export]
macro_rules! member_mutators {
    ($enum:ident, $variant:ident, $ty:ty, $add:ident, $remove:ident, $err:ident, $existing:ident) => {
        /// Append a node, erroring if one of that name is already present.
        pub fn $add(&mut self, value: $ty) -> Result<&mut $ty, $crate::FileError> {
            use $crate::node::AstNode as _;
            if let Some(existing) = self.$existing(&value.name) {
                return Err($crate::FileError::$err(existing.clone(), value));
            }
            self.mark_dirty();
            self.members.push($enum::$variant(value));
            match self.members.last_mut() {
                Some($enum::$variant(v)) => Ok(v),
                _ => unreachable!("just pushed this variant"),
            }
        }

        /// Remove the node of this name, returning it. `None` if absent.
        ///
        /// Marks the container dirty: its span still covers the text of the node
        /// that just left, so `Mode::Preserve` must re-print it rather than reuse
        /// the original bytes.
        pub fn $remove(&mut self, name: &str) -> Option<$ty> {
            use $crate::node::AstNode as _;
            let index = self.members.iter().position(|m| match m {
                $enum::$variant(v) => v.name == name,
                _ => false,
            })?;
            self.mark_dirty();
            match self.members.remove(index) {
                $enum::$variant(v) => Some(v),
                _ => unreachable!("index came from this variant"),
            }
        }
    };
}

/// Generates order-level operations shared by every container.
#[macro_export]
macro_rules! container_ops {
    ($enum:ident) => {
        pub fn member_count(&self) -> usize {
            self.members.len()
        }

        /// Append a member of any kind, keeping it last in source order.
        pub fn push_member(&mut self, member: $enum) {
            use $crate::node::AstNode as _;
            self.mark_dirty();
            self.members.push(member);
        }

        /// Insert at `index`, clamped to the end.
        pub fn insert_member_at(&mut self, index: usize, member: $enum) {
            use $crate::node::AstNode as _;
            let index = index.min(self.members.len());
            self.mark_dirty();
            self.members.insert(index, member);
        }

        pub fn remove_member_at(&mut self, index: usize) -> Option<$enum> {
            use $crate::node::AstNode as _;
            if index < self.members.len() {
                self.mark_dirty();
                Some(self.members.remove(index))
            } else {
                None
            }
        }

        /// Move the member at `from` to `to`. Out-of-range indices are a no-op.
        ///
        /// Because order is intrinsic to the member list, this is a plain
        /// `remove`+`insert` — there are no sidecar indices to repair (§1.1).
        pub fn move_member(&mut self, from: usize, to: usize) -> bool {
            use $crate::node::AstNode as _;
            if from >= self.members.len() || to >= self.members.len() {
                return false;
            }
            self.mark_dirty();
            let member = self.members.remove(from);
            self.members.insert(to, member);
            true
        }
    };
}

/// A node's children, in source order.
///
/// Children *should* already be ordered — the grammar is a sequence and
/// `BasicPublisher::connect` appends — but PEG backtracking and `connect_front`
/// make that an assumption rather than a guarantee. Sorting is cheap insurance.
/// Every comment inside a leaf node, in source order.
///
/// For leaf nodes — those with no ordered member list — this is the only place a
/// comment inside the node can be kept. Discarding them silently loses source
/// content, which the round-trip guarantee does not permit.
///
/// The search recurses, because grammar rules nest: a comment in
/// `version { major 25 // why` is a child of `major`, not of `version`. It stops
/// at `annotation_block`, whose comments belong to the annotations themselves and
/// would otherwise be captured twice.
pub(crate) fn trailing_comments(
    source: &str,
    publisher: &BasicPublisher,
    node: &Node,
) -> Vec<Comment> {
    let mut out = Vec::new();
    collect_comments(source, publisher, node, &mut out);
    out
}

fn collect_comments(source: &str, publisher: &BasicPublisher, node: &Node, out: &mut Vec<Comment>) {
    for child in sorted_children(publisher, node) {
        if child.rule == Rules::annotation_block {
            continue;
        }
        match Comment::from_cst(source, child) {
            Some(comment) => out.push(comment),
            None => collect_comments(source, publisher, child, out),
        }
    }
}

pub(crate) fn sorted_children<'a>(publisher: &'a BasicPublisher, node: &Node) -> Vec<&'a Node> {
    let mut children: Vec<&Node> = node
        .get_children()
        .iter()
        .map(|key| publisher.get_node(*key))
        .collect();
    children.sort_by_key(|c| c.start_position);
    children
}

/// Builds a container's ordered member list, applying the trivia-binding rule.
///
/// A run of comments that is followed immediately — with no blank line — by a
/// member becomes that member's `leading_comments`. Anything else stays a
/// free-floating `Comment` member at its original position.
pub(crate) struct MemberBuilder<'a, M: MemberEnum> {
    source: &'a str,
    members: Vec<M>,
    /// Comments not yet known to be free-floating or leading.
    pending: Vec<Comment>,
    /// Blank lines that preceded the first pending comment.
    pending_blank_lines: u8,
    /// End offset of the last node consumed, for blank-line arithmetic.
    cursor: u32,
    /// Comments seen before the body opened, e.g. `interface /* x */ Foo {`.
    header: Vec<Comment>,
    body_open: bool,
}

impl<'a, M: MemberEnum> MemberBuilder<'a, M> {
    /// `body_open` starts false for brace-delimited containers, which call
    /// [`Self::open_body`] when they reach their `open_bracket`. Containers with
    /// no braces (the file itself, parameter lists) pass `true`.
    pub fn new(source: &'a str, start: u32, body_open: bool) -> Self {
        Self {
            source,
            members: Vec::new(),
            pending: Vec::new(),
            pending_blank_lines: 0,
            cursor: start,
            header: Vec::new(),
            body_open,
        }
    }

    pub fn open_body(&mut self, node: &Node) {
        self.body_open = true;
        self.cursor = node.end_position;
    }

    /// Record a `Rules::comment` / `Rules::multiline_comment` child.
    pub fn comment(&mut self, node: &Node) {
        let Some(mut comment) = Comment::from_cst(self.source, node) else {
            return;
        };
        let blanks = count_blank_lines(self.source, self.cursor, node.start_position);
        self.cursor = node.end_position;

        if !self.body_open {
            comment.blank_lines_before = blanks;
            self.header.push(comment);
            return;
        }

        // A blank line separates this comment from whatever was pending, so the
        // pending run cannot be leading trivia for what follows.
        if blanks > 0 && !self.pending.is_empty() {
            self.flush_pending();
        }
        if self.pending.is_empty() {
            self.pending_blank_lines = blanks;
        } else {
            comment.blank_lines_before = blanks;
        }
        self.pending.push(comment);
    }

    /// Attach layout metadata to a node without adding it to the member list.
    /// Used for fixed-position children such as an interface's `version`.
    pub fn attach<T: AstNode>(&mut self, node: &Node, mut value: T) -> T {
        let blanks = self.consume(node, &mut value);
        value.meta_mut().blank_lines_before = blanks;
        value
    }

    /// Add a member, binding any pending comments to it.
    pub fn member<T: AstNode>(&mut self, node: &Node, mut value: T, wrap: impl FnOnce(T) -> M) {
        let blanks = self.consume(node, &mut value);
        value.meta_mut().blank_lines_before = blanks;
        self.members.push(wrap(value));
    }

    /// Shared by `attach` and `member`: binds pending comments and advances the
    /// cursor. Returns the blank-line count to record on the node.
    fn consume<T: AstNode>(&mut self, node: &Node, value: &mut T) -> u8 {
        let blanks = count_blank_lines(self.source, self.cursor, node.start_position);
        self.cursor = node.end_position;

        if blanks == 0 && !self.pending.is_empty() {
            // The pending run sits directly above this node: it is leading trivia.
            // The node inherits the spacing that preceded the run.
            value.meta_mut().leading_comments = std::mem::take(&mut self.pending);
            let run_blanks = self.pending_blank_lines;
            self.pending_blank_lines = 0;
            run_blanks
        } else {
            self.flush_pending();
            blanks
        }
    }

    fn flush_pending(&mut self) {
        let mut blanks = self.pending_blank_lines;
        for mut comment in self.pending.drain(..) {
            if blanks > 0 {
                comment.blank_lines_before = blanks;
                blanks = 0;
            }
            self.members.push(M::from_comment(comment));
        }
        self.pending_blank_lines = 0;
    }

    /// Consume the builder, flushing any trailing comments as free-floating
    /// members. Returns the members and the header comments.
    pub fn finish(mut self) -> (Vec<M>, Vec<Comment>) {
        self.flush_pending();
        (self.members, self.header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_ids_are_unique_and_nonzero() {
        let mut gen = NodeIdGen::new();
        let a = gen.next_id();
        let b = gen.next_id();
        assert_ne!(a, b);
        assert!(a.is_assigned());
        assert!(b.is_assigned());
        assert!(!NodeId::UNASSIGNED.is_assigned());
    }

    #[test]
    fn span_text_slices_source() {
        let src = "package a.b.c";
        assert_eq!(Span::new(8, 13).text(src), "a.b.c");
        assert_eq!(Span::new(8, 13).len(), 5);
    }

    #[test]
    fn comment_round_trips_delimiters() {
        let line = Comment::line(" hello");
        assert_eq!(line.to_source(), "// hello");
        let block = Comment::block(" hello ");
        assert_eq!(block.to_source(), "/* hello */");
    }

    #[test]
    fn comment_eq_ignores_layout_but_layout_eq_does_not() {
        let a = Comment::line(" x");
        let mut b = Comment::line(" x");
        b.blank_lines_before = 2;
        b.span = Some(Span::new(10, 14));
        assert_eq!(a, b);
        assert!(!a.eq_with_layout(&b));
    }

    #[test]
    fn blank_line_counting() {
        // "}\n  interface" -> adjacent lines, no blank line between.
        assert_eq!(count_blank_lines("}\n  interface", 1, 4), 0);
        // One fully empty line between.
        assert_eq!(count_blank_lines("}\n\n  interface", 1, 5), 1);
        // Two empty lines.
        assert_eq!(count_blank_lines("}\n\n\n  interface", 1, 6), 2);
        // Same line.
        assert_eq!(count_blank_lines("} interface", 1, 2), 0);
        // Degenerate ranges are not an error.
        assert_eq!(count_blank_lines("abc", 2, 2), 0);
        assert_eq!(count_blank_lines("abc", 2, 99), 0);
    }
}
