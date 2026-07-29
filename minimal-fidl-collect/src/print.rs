//! Rendering the AST back to `.fidl` text. See `DESIGN.md` §8.
//!
//! Two modes, no dials. [`Mode::Format`] lays everything out from the tree in one
//! consistent style. [`Mode::Preserve`] emits any untouched subtree as the exact
//! bytes it was read from, so an edit produces a minimal diff.

use std::fmt;

use crate::{
    enumeration::EnumMember,
    fidl_file::FileMember,
    interface::InterfaceMember,
    method::{ParamList, ParamMember},
    node::{Comment, NodeMeta},
    node_ref::NodeRef,
    structure::StructMember,
    type_collection::TypeCollectionMember,
    Annotation, Attribute, EnumValue, Enumeration, FidlFile, ImportModel, ImportNamespace,
    Interface, Method, Package, Structure, TypeCollection, TypeDef, VariableDeclaration, Version,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// One opinionated style, laid out from the tree. Blank-line runs collapse to
    /// one. The default.
    #[default]
    Format,
    /// Untouched subtrees are emitted verbatim from their original span; only
    /// modified or synthesised nodes are formatted.
    Preserve,
}

const INDENT: &str = "    ";

impl FidlFile {
    /// Render the file using [`Mode::Format`].
    pub fn to_fidl(&self) -> String {
        self.to_fidl_with(Mode::Format)
    }

    pub fn to_fidl_with(&self, mode: Mode) -> String {
        let mut printer = Printer {
            source: &self.source,
            mode,
            lines: Vec::new(),
            indent: 0,
        };
        printer.file(self);
        let mut out = printer.lines.join("\n");
        // Exactly one trailing newline, whatever the tree held.
        while out.ends_with('\n') {
            out.pop();
        }
        out.push('\n');
        out
    }
}

impl fmt::Display for FidlFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_fidl())
    }
}

struct Printer<'a> {
    source: &'a str,
    mode: Mode,
    lines: Vec<String>,
    indent: usize,
}

impl<'a> Printer<'a> {
    fn push(&mut self, text: impl AsRef<str>) {
        self.lines
            .push(format!("{}{}", INDENT.repeat(self.indent), text.as_ref()));
    }

    /// A blank line, unless one is already pending or nothing has been written.
    fn blank(&mut self) {
        match self.lines.last() {
            None => return,
            Some(last) if last.is_empty() => return,
            Some(_) => self.lines.push(String::new()),
        }
    }

    /// Emit `blank_lines_before`, clamped to one. Suppressed at the very start of
    /// a block so containers never open with a stray blank line.
    fn spacing(&mut self, meta: &NodeMeta) {
        if meta.blank_lines_before > 0 {
            self.blank();
        }
    }

    /// Multi-line text keeps its original interior layout; only the first line is
    /// re-indented. Used for verbatim spans and block comments, where reflowing
    /// would alter content.
    fn push_raw_block(&mut self, text: &str) {
        let mut parts = text.split('\n');
        if let Some(first) = parts.next() {
            self.push(first);
        }
        for rest in parts {
            self.lines.push(rest.to_string());
        }
    }

    /// In `Preserve` mode, emit an untouched subtree exactly as it was read.
    ///
    /// A subtree qualifies only if every node in it is clean and the root still
    /// has a span; anything synthesised or modified has no original text to reuse.
    fn try_verbatim(&mut self, node: NodeRef<'_>) -> bool {
        if self.mode != Mode::Preserve {
            return false;
        }
        let Some(span) = node.span() else {
            return false;
        };
        let dirty = node
            .self_and_descendants()
            .any(|n| n.meta().is_some_and(|m| m.dirty));
        if dirty {
            return false;
        }
        let text = span.text(self.source);
        // A span starts at the node's first character, so it excludes the
        // indentation in front of it. Recover that from the source, otherwise
        // every member's first line gets re-indented to the printer's level and
        // Preserve stops being byte-exact for any file that is not already
        // formatted at column zero.
        match self.original_indent(span.start) {
            Some(indent) => {
                let mut parts = text.split('\n');
                if let Some(first) = parts.next() {
                    self.lines.push(format!("{indent}{first}"));
                }
                for rest in parts {
                    self.lines.push(rest.to_string());
                }
            }
            None => self.push_raw_block(text),
        }
        true
    }

    /// The whitespace between the start of the line and `offset`, when the node
    /// is the first thing on its line. `None` if anything else precedes it.
    fn original_indent(&self, offset: u32) -> Option<&'a str> {
        let offset = offset as usize;
        let line_start = self.source[..offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &self.source[line_start..offset];
        prefix.chars().all(|c| c == ' ' || c == '\t').then_some(prefix)
    }

    /// Append text to the line just written, for same-line trailing comments.
    fn append_to_last(&mut self, text: &str) {
        match self.lines.last_mut() {
            Some(last) => last.push_str(text),
            None => self.push(text.trim_start()),
        }
    }

    /// Comments that sat inside the node after its content. The first goes on the
    /// same line; the rest follow it, since a `//` comment swallows its line.
    fn trailing(&mut self, meta: &NodeMeta) {
        let mut comments = meta.trailing_comments.iter();
        if let Some(first) = comments.next() {
            self.append_to_last(&format!(" {}", first.to_source()));
        }
        for rest in comments {
            self.push_raw_block(&rest.to_source());
        }
    }

    fn comment(&mut self, comment: &Comment) {
        if comment.blank_lines_before > 0 {
            self.blank();
        }
        self.push_raw_block(&comment.to_source());
    }

    /// Emit one member: spacing, leading trivia, then either its original bytes
    /// or a formatted rendering.
    ///
    /// The split matters. Leading comments sit *outside* the node's span — they
    /// come from the parent's whitespace — so they are always printed here.
    /// Annotations and header comments are *inside* it, because `annotation_block`
    /// is the node's own first child, so the verbatim path must not print them
    /// again or they appear twice.
    fn member(
        &mut self,
        meta: &NodeMeta,
        annotations: &[Annotation],
        node: NodeRef<'_>,
        render: impl FnOnce(&mut Self),
    ) {
        self.spacing(meta);
        for comment in &meta.leading_comments {
            self.comment(comment);
        }
        if self.try_verbatim(node) {
            return;
        }
        for comment in &meta.header_comments {
            self.push_raw_block(&comment.to_source());
        }
        self.annotations(annotations);
        render(self);
        // Version emits its own inside the braces, where they were written.
        if !matches!(node, NodeRef::Version(_)) {
            self.trailing(meta);
        }
    }

    fn annotations(&mut self, annotations: &[Annotation]) {
        if annotations.is_empty() {
            return;
        }
        // Comments that were written inside the block are emitted *above* it.
        //
        // `annotation_content` matches everything up to the next `@` or `**>`, so
        // a comment left inside the braces is swallowed into the preceding
        // annotation's content on the next parse and stops being a comment at all.
        // Hoisting them out keeps the text and keeps printing idempotent; the only
        // cost is that they move above the block.
        for annotation in annotations {
            for comment in annotation
                .meta
                .leading_comments
                .iter()
                .chain(&annotation.meta.trailing_comments)
            {
                self.push_raw_block(&comment.to_source());
            }
        }

        if let [single] = annotations {
            self.push(format!(
                "<** @{}:{} **>",
                single.name,
                trailing_space_trimmed(&single.contents)
            ));
            return;
        }
        self.push("<**");
        self.indent += 1;
        for annotation in annotations {
            self.push(format!(
                "@{}:{}",
                annotation.name,
                trailing_space_trimmed(&annotation.contents)
            ));
        }
        self.indent -= 1;
        self.push("**>");
    }

    /// Open a braced body, run `body`, close it. An empty body collapses to `{}`.
    fn block(&mut self, header: String, is_empty: bool, body: impl FnOnce(&mut Self)) {
        if is_empty {
            self.push(format!("{header} {{}}"));
            return;
        }
        self.push(format!("{header} {{"));
        self.indent += 1;
        let before = self.lines.len();
        body(self);
        // A body that opened with a blank line reads badly; drop it.
        if self.lines.get(before).is_some_and(|l| l.is_empty()) {
            self.lines.remove(before);
        }
        self.indent -= 1;
        while self.lines.last().is_some_and(|l| l.is_empty()) {
            self.lines.pop();
        }
        self.push("}");
    }

    fn file(&mut self, file: &FidlFile) {
        for comment in &file.meta.header_comments {
            self.comment(comment);
        }
        for member in &file.members {
            match member {
                FileMember::Comment(c) => self.comment(c),
                FileMember::Package(p) => {
                    self.member(&p.meta, &[], NodeRef::Package(p), |s| s.package(p))
                }
                FileMember::ImportNamespace(n) => self.member(
                    &n.meta,
                    &[],
                    NodeRef::ImportNamespace(n),
                    |s| s.import_namespace(n),
                ),
                FileMember::ImportModel(i) => {
                    self.member(&i.meta, &[], NodeRef::ImportModel(i), |s| s.import_model(i))
                }
                FileMember::Interface(iface) => self.member(
                    &iface.meta,
                    &iface.annotations,
                    NodeRef::Interface(iface),
                    |s| s.interface(iface),
                ),
                FileMember::TypeCollection(tc) => self.member(
                    &tc.meta,
                    &tc.annotations,
                    NodeRef::TypeCollection(tc),
                    |s| s.type_collection(tc),
                ),
            }
        }
    }

    fn package(&mut self, package: &Package) {
        self.push(format!("package {}", package.path.join(".")));
    }

    fn import_namespace(&mut self, ns: &ImportNamespace) {
        let wildcard = if ns.wildcard { ".*" } else { "" };
        self.push(format!(
            "import {}{} from \"{}\"",
            ns.import.join("."),
            wildcard,
            ns.from.display()
        ));
    }

    fn import_model(&mut self, model: &ImportModel) {
        self.push(format!("import model \"{}\"", model.file_path.display()));
    }

    fn interface(&mut self, iface: &Interface) {
        let empty = iface.version.is_none() && iface.members.is_empty();
        let header = format!("interface {}", iface.name);
        self.block(header, empty, |p| {
            if let Some(version) = &iface.version {
                p.member(&version.meta, &[], NodeRef::Version(version), |p| {
                    p.version(version)
                });
            }
            for member in &iface.members {
                match member {
                    InterfaceMember::Comment(c) => p.comment(c),
                    InterfaceMember::Method(m) => {
                        p.member(&m.meta, &m.annotations, NodeRef::Method(m), |p| p.method(m))
                    }
                    InterfaceMember::Attribute(a) => p.member(
                        &a.meta,
                        &a.annotations,
                        NodeRef::Attribute(a),
                        |p| p.attribute(a),
                    ),
                    InterfaceMember::Structure(st) => p.member(
                        &st.meta,
                        &st.annotations,
                        NodeRef::Structure(st),
                        |p| p.structure(st),
                    ),
                    InterfaceMember::Enumeration(e) => p.member(
                        &e.meta,
                        &e.annotations,
                        NodeRef::Enumeration(e),
                        |p| p.enumeration(e),
                    ),
                    InterfaceMember::TypeDef(t) => {
                        p.member(&t.meta, &t.annotations, NodeRef::TypeDef(t), |p| p.typedef(t))
                    }
                }
            }
        });
    }

    fn type_collection(&mut self, tc: &TypeCollection) {
        let empty = tc.version.is_none() && tc.members.is_empty();
        let header = if tc.is_anonymous() {
            "typeCollection".to_string()
        } else {
            format!("typeCollection {}", tc.name)
        };
        self.block(header, empty, |p| {
            if let Some(version) = &tc.version {
                p.member(&version.meta, &[], NodeRef::Version(version), |p| {
                    p.version(version)
                });
            }
            for member in &tc.members {
                match member {
                    TypeCollectionMember::Comment(c) => p.comment(c),
                    TypeCollectionMember::TypeDef(t) => {
                        p.member(&t.meta, &t.annotations, NodeRef::TypeDef(t), |p| p.typedef(t))
                    }
                    TypeCollectionMember::Structure(st) => p.member(
                        &st.meta,
                        &st.annotations,
                        NodeRef::Structure(st),
                        |p| p.structure(st),
                    ),
                    TypeCollectionMember::Enumeration(e) => p.member(
                        &e.meta,
                        &e.annotations,
                        NodeRef::Enumeration(e),
                        |p| p.enumeration(e),
                    ),
                }
            }
        });
    }

    fn version(&mut self, version: &Version) {
        let empty = version.major.is_none()
            && version.minor.is_none()
            && version.meta.trailing_comments.is_empty();
        self.block("version".to_string(), empty, |p| {
            if let Some(major) = version.major {
                p.push(format!("major {major}"));
            }
            if let Some(minor) = version.minor {
                p.push(format!("minor {minor}"));
            }
            // Kept inside the braces: that is where they were, and moving them out
            // would re-home them onto the enclosing interface on the next parse.
            for comment in &version.meta.trailing_comments {
                p.push_raw_block(&comment.to_source());
            }
        });
    }

    fn method(&mut self, method: &Method) {
        let empty = method.inputs.members.is_empty() && method.outputs.members.is_empty();
        self.block(format!("method {}", method.name), empty, |p| {
            if !method.inputs.members.is_empty() {
                p.param_list("in", &method.inputs);
            }
            if !method.outputs.members.is_empty() {
                p.param_list("out", &method.outputs);
            }
        });
    }

    fn param_list(&mut self, keyword: &str, list: &ParamList) {
        for comment in &list.meta.header_comments {
            self.push_raw_block(&comment.to_source());
        }
        self.annotations(&list.annotations);
        self.block(keyword.to_string(), list.members.is_empty(), |p| {
            for member in &list.members {
                match member {
                    ParamMember::Comment(c) => p.comment(c),
                    ParamMember::Param(param) => p.member(
                        &param.meta,
                        &param.annotations,
                        NodeRef::VariableDeclaration(param),
                        |p| p.variable_declaration(param),
                    ),
                }
            }
        });
        self.trailing(&list.meta);
    }

    fn structure(&mut self, structure: &Structure) {
        self.block(
            format!("struct {}", structure.name),
            structure.members.is_empty(),
            |p| {
                for member in &structure.members {
                    match member {
                        StructMember::Comment(c) => p.comment(c),
                        StructMember::Field(field) => p.member(
                            &field.meta,
                            &field.annotations,
                            NodeRef::VariableDeclaration(field),
                            |p| p.variable_declaration(field),
                        ),
                    }
                }
            },
        );
    }

    fn enumeration(&mut self, enumeration: &Enumeration) {
        self.block(
            format!("enumeration {}", enumeration.name),
            enumeration.members.is_empty(),
            |p| {
                for member in &enumeration.members {
                    match member {
                        EnumMember::Comment(c) => p.comment(c),
                        EnumMember::Value(value) => p.member(
                            &value.meta,
                            &value.annotations,
                            NodeRef::EnumValue(value),
                            |p| p.enum_value(value),
                        ),
                    }
                }
            },
        );
    }

    fn enum_value(&mut self, value: &EnumValue) {
        match value.value {
            Some(v) => self.push(format!("{} = {}", value.name, v)),
            None => self.push(&value.name),
        }
    }

    fn attribute(&mut self, attribute: &Attribute) {
        self.push(format!(
            "attribute {} {}",
            attribute.type_n, attribute.name
        ));
    }

    fn typedef(&mut self, typedef: &TypeDef) {
        let array = if typedef.is_array { "[]" } else { "" };
        self.push(format!(
            "typedef {} is {}{}",
            typedef.name, typedef.type_n, array
        ));
    }

    fn variable_declaration(&mut self, var: &VariableDeclaration) {
        let array = if var.is_array { "[]" } else { "" };
        self.push(format!("{}{} {}", var.type_n, array, var.name));
    }
}

/// Annotation contents keep the whitespace that followed the `:` in the source,
/// so `@name:` + contents reproduces `@name: value`. Trailing space would end up
/// before the closing `**>`.
fn trailing_space_trimmed(contents: &str) -> String {
    contents.trim_end().to_string()
}
