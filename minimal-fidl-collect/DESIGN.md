# minimal-fidl-collect: AST design

Status: **all phases (0–7) plus phase 9 implemented.** See §13.
Scope: turning the CST produced by `minimal-fidl-parser` into a mutable, ordered,
round-trippable AST for `.fidl` files.

### Implementation notes (things that differed from the plan)

- Rather than repeating `id` / `span` / `blank_lines_before` / `leading_comments`
  on fifteen structs, they live in a single `NodeMeta` embedded as a `meta` field,
  with an `AstNode` trait providing the accessors. `impl_ast_node!` generates the
  impls.
- `NodeMeta` gained a fifth field, `header_comments`, for comments inside a node's
  own header (`interface /* x */ Foo {`). Without it those comments have nowhere
  to live and would be dropped, breaking the preservation guarantee.
- Phases 1 and 2 could not be separated in practice: changing a container's shape
  forces its constructor to change in the same commit. They landed together.
- `Method`'s parameter lists are a `ParamList` node rather than a bare `Vec`, so
  that comments inside `in { ... }` have an owner.
- The dead `Rules::type_dec` arm in `Method::new` (§14) was confirmed unreachable
  and removed.
- No immutable `Visit` trait was written. `NodeRef` + `descendants()` covers
  reading; a second trait would have doubled the surface for nothing. `VisitMut`
  exists because `&mut` references cannot be gathered into a `Vec` the way
  `NodeRef` gathers shared ones.
- `VisitMut` carries an invariant worth knowing: every node must route through
  `self.visit_meta(&mut node.meta)`, never `walk_meta` directly. Leaf nodes
  originally called `walk_meta`, which silently skipped any `visit_meta` override
  — id assignment lost eight nodes to it before the Phase 3 acceptance test
  caught the duplicate ids.
- `NodeMeta::dirty` landed here rather than in Phase 5. Every mutation entry point
  has to set it, so adding it later would have meant revisiting all of them.
- `FidlFile::edit(|f| ...)` was added because `assign_missing_ids()` is easy to
  forget after a structural change, and a node with no id is invisible to
  `get()` and `path_of()`.
- **Duplicate names are a `validate()` diagnostic, not a construction error.**
  Phase 4 kept the old constructor-time rejection to avoid breaking seven tests
  that asserted it; Phase 5 reversed that, because it meant 13 of the 32 corpus
  files could not be loaded at all. That blocked the formatter delegation and —
  worse — meant the tool could not open a file in order to *fix* its duplicates.
  The seven tests now assert a diagnostic instead.
- `NodeMeta` gained `trailing_comments`. Leaf nodes (`package`, `version`,
  `attribute`, imports, typedefs, parameters, annotations) had no member list and
  were silently discarding every comment inside them — 12 of 32 corpus files lost
  comments, up to 53 of 83 in the worst case. The capture recurses, since a comment
  in `version { major 25 // why` is a child of `major`, not of `version`.
- `ParamList` gained `annotations`. `in {}` / `out {}` annotation blocks were being
  dropped outright, not just their comments.
- Anonymous `typeCollection { ... }` is now accepted (§14 resolved). The grammar
  allows it and the old formatter handled it; `validate()` warns that nothing can
  refer to it.
- The `#[pymodule]` macro cannot see pyclasses produced by a macro — it processes
  the module body before expansion — so the handle classes need an explicit
  `#[pymodule_init]`. Without it they work as return values but cannot be
  imported or used with `isinstance`.
- PyO3 rejects macro invocations inside `#[pymethods]`, so the `handle!` macro
  emits the whole block and takes the type's own methods as a token tree.
- A minimal Python mutation surface (`remove_method`, `remove_attribute`) landed
  because the stale-handle guard cannot be tested without one. Phase 9 replaced
  it with the full API.
- Those two methods had a live bug, and it is the reason `get_mut` exists.
  Lacking one, they read their own `name` back and called
  `file.interface_mut(&name)` — which returns the *first* interface of that name.
  Since duplicate names are only a `validate()` diagnostic, a handle to the second
  of two same-named interfaces mutated the first one instead. The stale-handle
  guard cannot catch it: the node it resolves is perfectly valid, just not the one
  that was asked for. **Resolve by id, never by name, whenever an id is in hand.**
- Removing a member now marks its container dirty. It did not before, and
  `Mode::Preserve` would happily re-emit the container from its span — removed
  member included. In practice the `*_mut()` accessor on the way in had already
  marked it, which is why no test saw this; going through `get_mut` made the gap
  reachable, so `member_mutators!` and `container_ops!` mark it themselves now.
- Two things the implemented grammar subset does not allow, discovered while
  writing builders for them, both now enforced by the builder rather than
  discovered by the user at reparse time:
  - `package` is **mandatory** (`grammar` sequences it without `_optional`), so a
    file with no package cannot be read back. `remove_package` exists to replace
    one, not to leave a file without one.
  - `import_namespace` **requires** the `.*`: `wildcard` is a mandatory element of
    the rule, not an option. `ImportNamespace::create` therefore always sets
    `wildcard: true`. The field stays public for anyone modelling something the
    parser will not accept.
- File-level insertion computes an index rather than appending. `grammar` is
  `package (import)* (interface | typeCollection)*` and enforces that order, so
  `push_member(FileMember::ImportModel(..))` would print the import after the
  interfaces and the output would not reparse.
- The diff's sibling match key carries an occurrence counter on *every* key, not
  just anonymous ones. Since duplicate names are legal enough to parse, two
  same-named interfaces otherwise collide in the map and a file reports
  differences against itself.
- `tests/real_files.rs` runs the whole pipeline over the repository's actual
  `.fidl` models rather than the hand-written parser fragments. It caught the one
  case the synthetic corpus could not: `Preserve` re-indenting the first line of
  every member, invisible in files already formatted at column zero.
- `reformatting_is_not_a_semantic_change` runs the differ over every corpus file
  against its own formatted output. It is the strongest single check in the suite:
  it ties the printer and the differ together and would catch either one silently
  altering meaning.

---

## 1. Motivation

`minimal-fidl-parser` produces a `BasicPublisher`: a flat arena of

```rust
Node { rule: Rules, start_position: u32, end_position: u32, result: bool, children: Vec<Key> }
```

This is a concrete syntax tree. It has no detachable text (`Node::get_string` needs
the original `&str`), no mutation API, and every grammar artefact is a node.

`minimal-fidl-collect` currently walks that CST into typed structs (`Interface`,
`Method`, `Attribute`, ...). Those structs are build-once and read-only, and they
lose three things we need:

1. **Order.** `Interface` holds `Vec<Method>`, `Vec<Attribute>`, `Vec<Structure>`,
   `Vec<Enumeration>`, `Vec<TypeDef>` in *separate* vectors. The source interleaving
   of members is unrecoverable. Same for `TypeCollection`, `Structure`, `FidlFileRs`.
2. **Comments.** Every collector has a `Rules::comment | Rules::multiline_comment => {}`
   arm. Writing a file back out would silently delete every comment in it.
3. **Everything else an AST is for** — traversal, mutation, printing, diffing.
   The only printer we have (`minimal-fidl-formatter`) formats from the *CST*, so it
   cannot print a tree you have edited.

### 1.1 Why not the `order: Vec<AttributeNodes>` sidecar

The in-progress `ordered.rs` / `AttributeNodes` approach (a `Vec` of variants holding
`u32` indices into parallel typed vectors) does not work, for two reasons:

- `trait Ordered { fn get_iterator(&self) -> impl Iterator<Item = &impl Ordered>; }`
  cannot express heterogeneous children. RPITIT means each impl returns exactly one
  concrete element type, but an interface's ordered children are
  method | attribute | struct | enum | typedef | comment. `dyn Ordered` is not
  object-safe with an RPITIT method — this is the current `E0038`.
  **Heterogeneous children need an enum, not a trait.**
- The sidecar's stated benefit (insertion without renumbering) only holds for
  *appends*. Any removal, or any insertion before the end, invalidates every index
  past that point, so a fixup pass is required regardless — and now two structures
  can silently desynchronise.

Both problems disappear if order is **intrinsic**: one ordered `Vec<Member>` where
`Member` is an enum. Insert/remove is `Vec::insert`/`Vec::remove`, nothing to
renumber, nothing to desync. `ordered.rs` and `AttributeNodes` are deleted.

---

## 2. Goals and non-goals

**Goals**

- Ordered, mutable node tree covering every `.fidl` construct.
- Comments and blank-line grouping preserved, so an unformatted file can be diffed
  without normalising it first.
- Uniform traversal: visit every node, read/modify every node's annotations.
- Structural mutation: add/remove/reorder methods, interfaces, type collections,
  structs, enums, typedefs, parameters.
- Structural diff between two trees.
- Read from file/string, write back to file.
- **Cheap for Python.** The PyO3 binding is the primary consumer; attribute access
  must not deep-copy subtrees.

**Non-goals (for now)**

- Byte-exact reproduction of whitespace *from the tree alone*. The tree stores
  comments and blank-line grouping, not indentation or incidental spacing, so
  `Mode::Format` output is normalised. `Mode::Preserve` gets byte-exactness for
  untouched subtrees by reusing their original source text, not by reconstructing
  it — see §8.
- Cross-file type resolution / import following. Deferred; the tree is designed to
  admit it later (§11).
- Error recovery in the parser. A file either parses or it does not.

---

## 3. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Ordered children as an **enum per container**, not parallel vectors + sidecar | §1.1 |
| D2 | Typed accessors (`methods()`) are **derived** from the ordered vector | Cannot desync with order |
| D3 | Every node carries a stable **`NodeId`** | Cheap, stable Python handles (§6) |
| D4 | `span: Option<Span>` — `None` means synthesised | Distinguish parsed from constructed |
| D5 | Comments are first-class; adjacent ones **bind** to the following member | `remove_method` should take its doc comment |
| D6 | `blank_lines_before: u8` on each member | Diff unformatted files; preserve grouping |
| D7 | Printing has exactly two modes: `Format` (default, opinionated) and `Preserve` (untouched subtrees emitted verbatim from their span) — no dials | Matches shipped formatter behaviour; gives minimal diffs on edit |
| D8 | `minimal-fidl-formatter` is **reversed** to delegate to the AST printer | Its 32 tests become the round-trip oracle |
| D9 | `FidlFileRs` renamed to `FidlFile`; Python import name unchanged | §10 |

---

## 4. Node model

### 4.1 Containers

Identity and layout live in a shared [`NodeMeta`](#43-spans-trivia-and-layout)
rather than being repeated on every struct.

```rust
pub struct FidlFile {
    pub meta: NodeMeta,
    pub source: String,             // spans are only meaningful against this
    pub path: Option<PathBuf>,      // set by from_path, used by save()
    pub members: Vec<FileMember>,
    ids: NodeIdGen,                 // NodeId allocator, private
}

pub enum FileMember {
    Package(Package),
    ImportNamespace(ImportNamespace),
    ImportModel(ImportModel),
    Interface(Interface),
    TypeCollection(TypeCollection),
    Comment(Comment),
}

pub struct Interface {
    pub meta: NodeMeta,
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub version: Option<Version>,
    pub members: Vec<InterfaceMember>,
}

pub enum InterfaceMember {
    Method(Method),
    Attribute(Attribute),
    Structure(Structure),
    Enumeration(Enumeration),
    TypeDef(TypeDef),
    Comment(Comment),
}
```

Same shape for:

- `TypeCollection { members: Vec<TypeCollectionMember> }` — typedef | struct | enum | comment
- `Structure { members: Vec<StructMember> }` — field | comment
- `Enumeration { members: Vec<EnumMember> }` — value | comment
- `Method { inputs: ParamList, outputs: ParamList }`, where
  `ParamList { meta, members: Vec<ParamMember> }` — param | comment. The list is a
  node in its own right so that comments inside `in { ... }` have an owner.

`version` stays a dedicated `Option<Version>` field rather than a member variant:
the grammar allows exactly one, in a fixed position, before other members.

### 4.2 Derived accessors

```rust
impl Interface {
    pub fn methods(&self) -> impl Iterator<Item = &Method>;
    pub fn methods_mut(&mut self) -> impl Iterator<Item = &mut Method>;
    pub fn method(&self, name: &str) -> Option<&Method>;
    pub fn method_mut(&mut self, name: &str) -> Option<&mut Method>;
    // ... same for attributes, structures, enumerations, typedefs
}
```

These filter `members`, so they can never disagree with source order. Generated by a
small declarative macro to avoid 30 hand-written near-identical bodies.

### 4.3 Spans, trivia, and layout

```rust
pub struct Span { pub start: u32, pub end: u32 }

/// Carried by every node as a `meta` field; reached through the `AstNode` trait.
pub struct NodeMeta {
    pub id: NodeId,
    pub span: Option<Span>,
    pub blank_lines_before: u8,
    /// Comments directly above the node, with no blank line. They travel with it.
    pub leading_comments: Vec<Comment>,
    /// Comments inside the node's own header: `interface /* x */ Foo {`.
    pub header_comments: Vec<Comment>,
}

/// Comments hold their own identity rather than a `NodeMeta`, since they cannot
/// themselves carry leading trivia.
pub struct Comment {
    pub id: NodeId,
    pub span: Option<Span>,
    pub blank_lines_before: u8,
    pub kind: CommentKind,   // Line | Block
    pub text: String,        // content only, delimiters excluded
}
```

- **Spans** are the *original* source range, valid only against `FidlFile::source`.
  They go stale the moment a node is mutated. Used for diagnostics and for
  `Mode::Preserve`'s "untouched subtree ⇒ reuse original text" (§8). Synthesised
  nodes get `None`.
- **Comments are already in the CST.** `wsn` in `parser.rs` emits `Rules::comment`
  and `Rules::multiline_comment` nodes, and `wsn` runs between every grammar
  element. No parser change is needed — collect is simply discarding them today.
- **Trivia binding rule (D5):** a comment immediately preceding a member with no
  intervening blank line becomes that member's `leading_comments`. Anything else
  stays a free-floating `Comment` member at its position. This is what makes
  `remove_method("play")` carry the method's doc comment with it.
- **Blank lines (D6):** `wsn` consumes blank lines without emitting nodes, so
  `blank_lines_before` is computed during construction by counting `\n` in the
  source gap between the previous member's `end` and this member's `start`. Stored
  saturating at 255 (a storage cap only; real files never approach it).
  `Mode::Format` collapses runs to one blank line, `Mode::Preserve` reuses the
  original text — see §8.
- **Child ordering:** CST children *should* already be in source order, but PEG
  backtracking and `BasicPublisher::connect_front` make that an assumption rather
  than a guarantee. Construction sorts each node's children by `start_position`
  before walking them. Cheap insurance.

### 4.4 Equality

`PartialEq` is hand-written (not derived) to **ignore `id`, `span`, and
`blank_lines_before`**, so a node that moved but did not change compares equal.
Layout-sensitive comparison is available separately via `Node::eq_with_layout`,
used by the diff engine's `ignore_layout: false` mode.

---

## 5. Traversal and addressing

```rust
pub enum NodeRef<'a> {
    File(&'a FidlFile), Interface(&'a Interface), Method(&'a Method),
    Attribute(&'a Attribute), Structure(&'a Structure), /* ... */ Comment(&'a Comment),
}

impl<'a> NodeRef<'a> {
    pub fn id(&self) -> NodeId;
    pub fn span(&self) -> Option<Span>;
    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a>>;
    pub fn descendants(&self) -> impl Iterator<Item = NodeRef<'a>>;  // pre-order
}
```

`NodeRef` is the object-safe replacement for the `Ordered` trait. `NodeRefMut`
mirrors it, with two differences that follow from `&mut` not being shareable:

```rust
pub enum NodeRefMut<'a> { File(&'a mut FidlFile), Interface(&'a mut Interface), /* ... */ }

impl<'a> NodeRefMut<'a> {
    pub fn children_mut(self) -> Vec<NodeRefMut<'a>>;   // takes self by value
    pub fn annotations_mut(&mut self) -> Option<&mut Vec<Annotation>>;
    pub fn mark_dirty(&mut self);
}

impl FidlFile {
    pub fn get_mut(&mut self, id: NodeId) -> Option<NodeRefMut<'_>>;
}
```

- It cannot be `Copy`, so `children_mut` takes `self` by value. Gathering a node's
  children into a `Vec` is possible despite the usual objection: they live in
  disjoint fields (`members`, `annotations`, the three comment lists on `meta`),
  and disjoint `&mut` borrows may coexist.
- **There is deliberately no mutable `Descendants`.** `VisitMut` covers whole-tree
  transforms and `get_mut` covers single-node lookup; between them nothing is
  missing, and a mutable iterator is the one genuinely awkward piece. `get_mut` is
  recursive descent rather than an iterator scan precisely because the borrow has
  to move down the tree and stop at the match.

`get_mut` marks the node it returns dirty, on the same conservative rule as the
`*_mut()` accessors. It does *not* mark the ancestors it descended through:
`Mode::Preserve` already refuses to reuse a span whose subtree contains anything
dirty, so marking the path would cost output fidelity for nothing.

`children()` and `children_mut()` must yield the same nodes in the same order.
`tests/node_ref_mut.rs` pins that against every node of a rich tree; without it a
mutable walk could silently skip the comments and annotations that live on `meta`
rather than in `members`.

```rust
pub trait Annotated {
    fn annotations(&self) -> &[Annotation];
    fn annotations_mut(&mut self) -> &mut Vec<Annotation>;
    fn annotation(&self, name: &str) -> Option<&Annotation>;
    fn set_annotation(&mut self, name: &str, contents: &str);
    fn remove_annotation(&mut self, name: &str) -> Option<Annotation>;
}
```

Implemented by every annotatable node. Unlike `Ordered`, this trait *is* object-safe
and earns its place — "walk the tree and rewrite every `@description`" is one pass.

`trait Visit` / `trait VisitMut` with defaulted walk methods (syn-style) for
whole-tree rewrites where a `descendants()` loop is too coarse.

**`NodePath`** — `[Interface("Foo"), Method("bar"), InputParam(0)]` — addresses a node
by name where names exist and by index where they do not. Used for diff output, error
messages, and as a stable-ish external reference in text form.

---

## 6. NodeId and the handle model

Every node carries `id: NodeId(u32)`, allocated from a per-file counter. IDs are:

- **stable** across sibling insertion, removal, and reordering;
- **never reused** within a file's lifetime;
- **not** stable across a reparse (a fresh parse allocates fresh IDs).

Resolution is `FidlFile::get(id) -> Option<NodeRef>`, implemented as a
`descendants()` scan, and `FidlFile::get_mut(id) -> Option<NodeRefMut>` by recursive
descent. `.fidl` files are small (hundreds of nodes), so this is negligible; if
profiling disagrees, add a lazily-rebuilt `HashMap<NodeId, NodePath>` behind a
dirty flag. **Do not build the index speculatively** — gate it on a criterion
benchmark (§12, Phase 8).

**Resolve by id, not by name, whenever an id is in hand.** Duplicate names are
legal enough to parse (§3), so `interface_mut("A")` answers "the first interface
called A", which is not the same question as "the interface this handle names".
Getting those two confused is a bug that no stale-handle guard can catch, because
the node it resolves is valid — just the wrong one.

This exists for the Python binding (§10): a handle is `(Arc<RwLock<FidlFile>>, NodeId)`,
two words, and stays valid when siblings change.

---

## 7. Mutation and validation

```rust
iface.add_method(
    Method::builder("play")
        .input("track", "UInt32")
        .output("ok", "Boolean")
        .annotation("description", "Start playback")
        .build()
)?;

iface.remove_method("stop");                  // -> Option<Method>, takes leading comments
iface.insert_member_at(3, InterfaceMember::Comment(Comment::line(" TODO")));
iface.move_member(from, to);
file.add_interface(iface)?;
```

Checking happens at two levels, and they are deliberately different:

- **Parsing is strict.** A `.fidl` with duplicate names fails to build a tree at
  all, as it always has. `add_*` matches that: it returns `Err` rather than
  admitting a duplicate.
- **`file.validate() -> Vec<Diagnostic>` is advisory and exhaustive.** It reports
  every problem it finds instead of stopping at the first, so a bulk edit can pass
  through illegal intermediate states and be checked once at the end. Reach it via
  the raw ordered list (`push_member`) when you want to defer checking.

Currently checked: duplicate sibling names, empty names, incomplete versions.
*Not* checked: unresolved type references. Doing that properly needs cross-file
import resolution (a §2 non-goal); without it every file that imports a type would
report false errors. It belongs with the §11 import work.

Two ergonomic points:

- Builders set `span: None` and `dirty: true` — a synthesised node has no original
  text, so `Mode::Preserve` must format it.
- Inserted nodes carry `NodeId::UNASSIGNED` until `assign_missing_ids()` runs, and
  a node with no id is invisible to `get()` and `path_of()`. `file.edit(|f| ...)`
  runs the closure and then assigns, which is harder to forget:

```rust
file.edit(|f| {
    f.interface_mut("Greeter")?.add_method(Method::builder("stop").build())
});
```

---

## 8. Printing

Two modes, no dials. Either the output is formatted to one opinionated style, or
it is left as the tree found it.

```rust
impl FidlFile {
    pub fn to_fidl(&self) -> String;                 // Mode::Format
    pub fn to_fidl_with(&self, mode: Mode) -> String;
}
impl fmt::Display for FidlFile { /* to_fidl() */ }

pub enum Mode {
    /// Opinionated: one style, blank-line runs collapse to one, everything
    /// re-laid-out from the tree. The default (D7).
    Format,
    /// Leave untouched subtrees exactly as they were read.
    Preserve,
}
```

`Format` needs no configuration — the style is whatever `minimal-fidl-formatter`
already emits, which is why its 32 tests are the oracle.

`Preserve` is implemented by **span reuse**: any node that still has a `span` and
has not been modified is emitted as `span.text(&self.source)`, byte for byte —
including its original indentation, which is recovered from the source because a
span starts at the node's first character and so excludes it.
Only modified or synthesised nodes are formatted. That makes an edit produce a
minimal diff — change one method and the rest of the file is untouched — which
matters for a tool meant to run over files people also hand-edit.

This requires a `dirty: bool` on `NodeMeta`, set by the Phase 4 mutation API and
by any `*_mut()` accessor handing out a mutable reference. Nodes with no span
(synthesised) are always formatted, since there is no original text to reuse.

Note what `Preserve` cannot do: a node whose *children* changed must be
re-emitted, so its incidental whitespace is normalised even though its untouched
siblings' is not. Fidelity is per-subtree, not global. At the file level it also
normalises leading blank lines and the trailing newline, which the printer owns
in either mode.

`tests/real_files.rs` pins this: an unedited file from the repository's own
corpus comes back byte for byte, edges aside.

Implementation reuses `IndentedString` from `minimal-fidl-formatter` so output style
matches what already ships. Once the AST printer exists, `Formatter::format()` is
rewritten as `parse -> AST -> to_fidl()` and the 1119-line CST walker in
`formatter.rs` is deleted (D8).

**The formatter's 32 tests are not an oracle.** Every one is
`parse -> format -> unwrap -> println!` with no assertion, so they detect a panic
and nothing else — they cannot see an output change at all. Phase 5 therefore
captured the CST formatter's output as golden files *before* replacing it
(`minimal-fidl-formatter/tests/golden/`), and added the properties below, which
are what actually holds the printer honest.

Properties, all checked over the full 32-input corpus:

- `format(format(x)) == format(x)` — idempotent
- `parse(format(x))` reparses at all — output that cannot be read back means
  running `fmt` destroys the file
- tree shape (kind and name of every node) is unchanged across a round trip
- **every comment the *parser* found is in the tree.** Comparing comments
  before and after a round trip is not enough: a comment dropped during
  construction is already missing from both sides. The parser's own comment nodes
  are the ground truth.

---

## 9. Diff

```rust
pub struct DiffOptions {
    pub ignore_comments: bool,   // default false
    pub ignore_layout: bool,     // default true
    pub ignore_order: bool,      // default false
}

pub enum Change {
    Added    { path: NodePath, node: OwnedNode },
    Removed  { path: NodePath, node: OwnedNode },
    Modified { path: NodePath, field: &'static str, before: String, after: String },
    Moved    { path: NodePath, from: usize, to: usize },
}

pub fn diff(a: &FidlFile, b: &FidlFile, opts: &DiffOptions) -> Vec<Change>;
```

Matching key: name-based for named nodes (interface, method, attribute, struct, enum,
typedef, enum value, parameter); positional for anonymous ones (comments,
annotations).

Because layout lives in the tree (D6), two *unformatted* files can be diffed directly
— no normalisation pass first.

Intended payoff: `diff` across two revisions of a `.fidl` gives **API breaking-change
detection** (removed method, changed parameter type, narrowed enum) for near-free.

---

## 10. File and project IO

```rust
impl FidlFile {
    pub fn from_path(p: impl AsRef<Path>) -> Result<Self, FileError>;
    pub fn from_source(src: &str) -> Result<Self, FileError>;   // also impl FromStr
    pub fn write_to(&self, p: impl AsRef<Path>) -> io::Result<()>;
    pub fn save(&self) -> io::Result<()>;                       // uses self.path
}

pub struct FidlProject { pub files: Vec<FidlFile> }
impl FidlProject {
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, FileError>;
    pub fn save_all(&self) -> io::Result<()>;
}
```

`FidlProject` keeps the existing directory walk. Cross-file import resolution is
deferred but the shape admits it.

---

## 11. Python binding

The binding is the primary consumer, so it drives the handle model (D3, §6).

**Naming (D9).** The Rust type becomes `FidlFile`. The binding aliases on import, so
the Python-visible name is unchanged:

```rust
use minimal_fidl_collect::FidlFile as AstFile;

#[pyclass(name = "FidlFile", frozen)]
struct PyFidlFile { inner: Arc<RwLock<AstFile>> }

#[pyclass(name = "FidlInterface", frozen)]
struct PyInterface { file: Arc<RwLock<AstFile>>, id: NodeId }
```

**Cheapness.** Today `#[pyo3(get)]` on `Vec<FidlInterface>` means `file.interfaces`
deep-clones the entire subtree *on every attribute access*. Under the handle model:

```rust
#[pymethods]
impl PyInterface {
    #[getter]
    fn name(&self) -> PyResult<String> {
        let f = self.file.read().unwrap();
        Ok(f.interface(self.id)?.name.clone())          // one String
    }

    #[getter]
    fn methods(&self) -> PyResult<Vec<PyMethod>> {
        let f = self.file.read().unwrap();
        Ok(f.interface(self.id)?
            .methods()
            .map(|m| PyMethod { file: self.file.clone(), id: m.id })
            .collect())                                  // N two-word handles
    }
}
```

**The pyclasses stay `frozen`.** Their own fields (`file`, `id`) never change;
mutation goes through the `RwLock`. Frozen pyclasses are cheaper and do not need the
GIL for field access, so this is strictly better than unfreezing them.

`Arc<RwLock<AstFile>>` is `Send + Sync` (the tree is plain data), satisfying PyO3's
`pyclass` requirement.

**Stale handles.** If a node has been removed, `f.interface(self.id)` returns `None`
and the getter raises a Python `StaleNodeError`. This is the safety story for handles
and must be tested explicitly.

**Two families of class, and the split is the whole API.**

- `Fidl*` is a **handle** on a node already in a tree. Not constructible from
  Python; you get one by reading or by inserting.
- `New*` is a **description** of a node to create — plain constructible data
  belonging to no file. Insertion consumes one and returns the handle.

```python
from franca_idl import FidlFile, NewMethod, NewParameter

f = FidlFile("player.fidl")
iface = f.interfaces[0]

play = iface.add_method(NewMethod("play",
                                 inputs=[NewParameter("track", "UInt32")],
                                 outputs=[NewParameter("ok", "Boolean")]))
play.set_annotation("description", " start playback")
iface.remove_method("stop")

for node in f.nodes():
    if node.kind != "file" and node.annotation("deprecated"):
        print(node.node_path)

for change in old.diff(f):
    if change.change_type == "removed":
        print("breaking:", change)

f.save(preserve=True)   # untouched subtrees come back byte for byte
```

Decisions worth keeping:

- **Typed `New*` classes, not keyword arguments.** More work to build, but the call
  site says what it is constructing and the types show up in editors and type
  checkers. Rejected: `(name, type)` tuples and `**kwargs`, which degenerate into
  tuples of tuples once nodes nest; and detached handles that are either attached
  or owning, where every accessor grows a branch.
- **Containers take one ordered `members` list**, mixed `New*` types, rather than a
  list per kind. Member order is intrinsic to this tree (§1.1) and per-kind lists
  would discard the interleaving at construction time.
- **`NewParameter` covers struct fields too.** A field and a parameter are the same
  node in the grammar, so there is no `NewField`.
- **Insertion returns the handle, under one lock.** A node from a `New*` carries
  `NodeId::UNASSIGNED` and is invisible to `get()` until ids are handed out, so the
  handle cannot exist until after the edit. `edit_then(mutate, locate)` does the
  mutation, calls `assign_missing_ids()`, and then reads the id back out — all
  inside one write guard, rather than mutating and looking the node up again by
  name afterwards.
- **`diff` is one flat `FidlChange` class** with `change_type: str`, plus `path`,
  `kind` and `Optional` fields for the rest. This keeps the case it exists for a
  one-liner: `[c for c in changes if c.change_type == "removed"]`. Rejected:
  `list[str]` (useless for filtering) and one class per variant (most ceremony).
- **Paths are objects, not strings.** `FidlNodePath` and `FidlPathSegment` are
  exposed so `at_path` can resolve what `path_of` produced. There is no parser for
  the printed form. The getter is `node_path`, not `path`, because a package's
  `path` is its dotted name and that name was there first.
- **The stub is enforced.** `franca_idl.pyi` is still written by hand, but
  `tests/test_stub.py` walks it with `ast` and compares it to the module in both
  directions — a stub entry that does not exist is a lie, and a public member the
  stub omits is a gap.

Two PyO3 constraints shape the code and are easy to rediscover the hard way:

- Macro invocations are illegal inside `#[pymethods]`, so `handle!` emits the whole
  block and takes the type's own methods as a token tree. Its arms (`annotated`,
  `meta`, `bare`) delegate downwards so each level's methods are written once.
- `#[pymodule]` cannot see macro-generated pyclasses — it processes the module body
  before expansion — so they are registered by hand in `#[pymodule_init]`. Anything
  new must be added there or it works as a return value but cannot be imported or
  used with `isinstance`.

---

## 12. Downstream migration

The field-to-accessor change (`interface.methods` -> `interface.methods()`) is
mechanical but touches:

- `minimal-fidl-generator/src/codegen_rust.rs`
- `minimal-fidl-generator/src/codegen_py.rs`
- `minimal-fidl-generator/src/codegen_trait.rs`
- `minimal-fidl-generator/src/lib.rs`
- `minimal-fidl-python/src/lib.rs` (rewritten to the handle model)
- `minimal-fidl-cli/src/fmt.rs` (via the formatter change)

---

## 13. Phases

| Phase | Work | Acceptance |
|---|---|---|
| **0** ✅ | Delete `ordered.rs` + `AttributeNodes`; add `Span`, `NodeId`, `Comment`, layout-ignoring `PartialEq` | `cargo check` green |
| **1** ✅ | Member enums for file/interface/type-collection/struct/enum/method; derived accessors; `FidlFileRs` -> `FidlFile` | Types compile; accessors unit-tested |
| **2** ✅ | Rewrite collectors to build ordered members; capture comments; trivia binding; `blank_lines_before`; child sort | Parse a comment- and blank-line-heavy fidl; assert exact member order, all comments, correct grouping |
| **3** ✅ | `NodeRef`/`children`/`descendants`, `Annotated`, `VisitMut`, `NodePath`, `FidlFile::get(id)` | Collect every annotation in the tree in one pass; resolve every id |
| **4** ✅ | Mutation API, builders, `validate()`, `NodeMeta::dirty` | Add/remove method, interface, type collection, param round-trip; duplicate detection |
| **5** ✅ | `to_fidl()` + `Mode::{Format,Preserve}`; formatter delegates to AST; delete CST walker | Idempotency, reparse, and comment-preservation properties hold over the full corpus |
| **6** ✅ | Structural diff + `Change` model | Detects add/remove/modify/move; `ignore_comments` and `ignore_layout` behave; unformatted-vs-unformatted diff is clean |
| **7** ✅ | File/project IO; migrate generator + CLI; rewrite Python binding to handles | Whole workspace builds; existing Python tests pass; stale-handle test raises |
| **9** ✅ | `NodeRefMut` + `get_mut`; file-level package/import mutation and builders; the whole AST API exposed to Python (`New*` inputs, insertion, diff, paths, traversal, `Preserve` output) | The wrong-duplicate mutation is fixed and pinned; `children_mut` agrees with `children`; every node reachable after an insertion resolves by id; the stub matches the module |
| **8** | *Optional:* `NodeId` index if benchmarks justify it | criterion benchmark shows a real win first |

---

## 14. Risks and open questions

- **Comment placement fidelity.** Comments in unusual positions (mid-parameter-list,
  between an annotation block and its target) may bind awkwardly. Phase 2 must build
  a corpus of adversarial comment placements before the binding rule is fixed.
- **`Rules::type_dec` in `Method::new`.** `method.rs` assigns `name` from both a
  `Rules::variable_name` and a `Rules::type_dec` arm, but the `method` grammar rule
  (`parser.rs:787`) only ever produces `variable_name` — `type_dec` belongs to
  `typedef`. The arm is dead code. Confirm and drop it in Phase 2 rather than
  carrying it forward.
- **Empty type-collection names.** `TypeCollection::new` errors when the name is
  empty, but its own comment says the name can legitimately be absent. Decide whether
  anonymous type collections are valid; if so, `name` becomes `Option<String>`.
- **`NodeId` across reparse.** IDs are not stable across a reparse, so a Python script
  holding handles across `f.reload()` gets stale-node errors. Acceptable, but must be
  documented prominently in the Python API docs.
- **Lock granularity.** One `RwLock` per file means a Python script mutating two
  interfaces in the same file serialises. Fine at current scale; revisit only if it
  shows up.
