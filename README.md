# lexbor_css — Safe, Fast CSS Selection for Rust

[![Rust](https://img.shields.io/badge/rust-stable-blue)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

A thin, safe Rust wrapper over the [lexbor](https://github.com/lexbor/lexbor) C library for
HTML parsing and CSS selection. Comparable to Python's [selectolax](https://github.com/rushter/selectolax)
lexbor backend — but in Rust, with zero overhead.

**62 tests, all passing.**

## Features

- **HTML parsing** — full HTML5 document parsing via lexbor's HTML parser
- **CSS selection** — full CSS3 selector support: tag, class, id, attribute,
  pseudo-class (`:not`, `:nth-child`, etc.), combinators, and compound selectors
- **DOM traversal** — parent, children, siblings, descendants, ancestors, depth-first traverse
- **DOM mutation** — decompose, append text, set/remove attributes
- **Node properties** — tag name, tag ID, attributes, classes, text content
- **Serialization** — inner/outer HTML with configurable pretty-printing options
- **Pre-compiled selectors** — compile once, reuse many times
- **Thread safety** — `!Send` + `!Sync` enforced at compile time (lexbor is single-threaded)
- **Safety** — all `unsafe` blocks isolated and documented; FFI callbacks wrapped in `catch_unwind`

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
lexbor_css = { git = "https://github.com/zacharyvmm/lexbor_css" }
```

The `vendor` feature is enabled by default and clones and builds lexbor from
source automatically. Requires `git`, `cmake`, and a C compiler.

Alternatively, install lexbor system-wide and disable the default features:

```toml
[dependencies]
lexbor_css = { git = "https://github.com/zacharyvmm/lexbor_css", default-features = false }
```

### Parsing and Selection

```rust
use lexbor_css::HtmlDocument;

let doc = HtmlDocument::parse(r#"
    <div class="content">
        <p>First paragraph</p>
        <p>Second paragraph</p>
        <span>Some span</span>
    </div>
"#)?;

// Select all matching nodes
let nodes = doc.select("div.content p");
assert_eq!(nodes.len(), 2);
assert_eq!(nodes[0].text(), "First paragraph");

// Select only the first match
let first = doc.select_first(".content span").unwrap();
assert_eq!(first.text(), "Some span");

// Pre-compile a selector for repeated use
let sel = doc.compile_selector("p")?;
for node in sel.find(&doc) {
    println!("{}", node.text());
}
```

### Advanced Selectors

```rust
// Attribute selectors
doc.select(r#"a[href="/home"]"#);
doc.select(r#"a[href^="/a"]"#);

// Pseudo-classes
doc.select("p:not(.x)");
doc.select("li:nth-child(2n)");

// Combinators
doc.select("h1 + p");       // adjacent sibling
doc.select("div, span");    // multiple selectors
doc.select("div#main p.intro");  // compound
```

### DOM Traversal

```rust
let div = doc.select_first("div.content").unwrap();

// Children
for child in div.children() {
    println!("tag={}", child.tag_name());
}

// Descendants (depth-first)
for node in div.descendants().filter(|n| n.is_element()) {
    println!("{}", node.tag_name());
}

// Traverse (includes starting node)
for node in div.traverse() {
    println!("{}", node.text());
}

// Siblings and ancestors
let parent = node.parent();
let next = node.next_sibling();
let prev = node.prev_sibling();
let ancestors: Vec<_> = node.ancestors().map(|n| n.tag_name()).collect();
```

### Node Properties

```rust
let node = doc.select_first("a").unwrap();

// Basic info
node.tag_name();          // "a"
node.tag_id();            // internal integer ID
node.is_element();        // true
node.is_text();           // false

// Attributes
node.attr("href");        // Some("/home")
node.has_attr("target");  // true
node.id();                // None
node.class();             // None
node.has_class("active"); // false
node.classes();           // Vec<&str>
node.attributes();        // HashMap<&str, &str>

// Text content
node.text();              // all descendant text
node.text_content();      // direct text only (None for elements)
node.text_with(TextOpts { deep: false, strip: true, ..Default::default() });
```

### Serialization

```rust
node.outer_html();   // "<a href=\"/home\">Home</a>"
node.inner_html();   // "Home"

// Pretty printing
let opts = SerializeOpts {
    indent: 2,
    skip_ws_nodes: true,
    skip_comment: true,
    ..Default::default()
};
node.outer_html_pretty(opts);
node.inner_html_pretty(opts);
```

### DOM Mutation

```rust
// Remove elements
let p = doc.select_first("p").unwrap();
p.decompose().unwrap();          // removes node and children
p.decompose_shallow().unwrap();  // removes node, keeps children
p.remove().unwrap();             // alias for decompose

// Modify attributes
node.set_attr("id", Some("new-id")).unwrap();
node.set_attr("disabled", None).unwrap();  // boolean attribute
node.remove_attr("class").unwrap();

// Add text content
let div = doc.select_first("div").unwrap();
div.append_text("Hello").unwrap();
div.append_text(" World").unwrap();
assert_eq!(div.text(), "Hello World");

// Strip tags by name
doc.strip_tags(&["script", "style"]);
```

### Document-Level Operations

```rust
let doc = HtmlDocument::parse(html)?;

// Access root/head/body
doc.root();  // <html> element node
doc.head();  // Some(Node) or None
doc.body();  // Some(Node) or None

// Fast tag lookup
let all_divs = doc.tags("div");

// Check if selector matches anywhere
assert!(doc.matches("p.intro"));

// CSS match on specific node
let node = doc.select_first("div").unwrap();
assert!(node.css_matches("div.main"));
assert!(node.any_css_matches(&["div", "span"]));
```

## API Reference

### HtmlDocument

| Method | Returns | Description |
|---|---|---|
| `parse(html)` | `Result<Self, Error>` | Parse full HTML document |
| `select(selector)` | `Vec<Node>` | All nodes matching CSS selector |
| `select_first(selector)` | `Option<Node>` | First matching node |
| `select_from(root, selector)` | `Vec<Node>` | Scoped selection from a root node |
| `compile_selector(s)` | `Result<Selector, Error>` | Pre-compile selector |
| `tags(name)` | `Vec<Node>` | Elements by tag name |
| `matches(selector)` | `bool` | Whether any node matches |
| `strip_tags(tags)` | — | Remove elements by tag name |
| `root()` | `Node` | Document root element |
| `head()` | `Option<Node>` | `<head>` element |
| `body()` | `Option<Node>` | `<body>` element |

### Node\<'a\>

#### Type Checks
| Method | Returns |
|---|---|
| `is_element()` | `bool` |
| `is_text()` | `bool` |
| `is_comment()` | `bool` |
| `is_document()` | `bool` |
| `is_empty_text_node()` | `bool` |
| `node_type()` | `u32` |

#### Traversal
| Method | Returns |
|---|---|
| `parent()` | `Option<Node>` |
| `first_child()` | `Option<Node>` |
| `last_child()` | `Option<Node>` |
| `next_sibling()` | `Option<Node>` |
| `prev_sibling()` | `Option<Node>` |
| `children()` | `impl Iterator<Item=Node>` |
| `descendants()` | `impl Iterator<Item=Node>` |
| `ancestors()` | `impl Iterator<Item=Node>` |
| `traverse()` | `impl Iterator<Item=Node>` |
| `iter_children(include_text, skip_empty)` | `impl Iterator<Item=Node>` |

#### Properties
| Method | Returns |
|---|---|
| `tag_name()` | `String` |
| `tag_id()` | `usize` |
| `attr(name)` | `Option<&str>` |
| `has_attr(name)` | `bool` |
| `id()` | `Option<&str>` |
| `class()` | `Option<&str>` |
| `has_class(name)` | `bool` |
| `classes()` | `Vec<&str>` |
| `attributes()` | `HashMap<&str, &str>` |

#### Text
| Method | Returns |
|---|---|
| `text()` | `String` |
| `text_with(opts)` | `String` |
| `text_content()` | `Option<&str>` |
| `comment_content()` | `Option<&str>` |

#### Serialization
| Method | Returns |
|---|---|
| `outer_html()` | `String` |
| `inner_html()` | `String` |
| `outer_html_pretty(opts)` | `String` |
| `inner_html_pretty(opts)` | `String` |

#### Mutation
| Method | Returns |
|---|---|
| `decompose()` | `Result<(), Error>` |
| `decompose_shallow()` | `Result<(), Error>` |
| `remove()` | `Result<(), Error>` |
| `append_text(text)` | `Result<(), Error>` |
| `set_attr(name, value)` | `Result<(), Error>` |
| `remove_attr(name)` | `Result<(), Error>` |

#### Matching
| Method | Returns |
|---|---|
| `css_matches(selector)` | `bool` |
| `any_css_matches(selectors)` | `bool` |

### Selector\<'a\>

| Method | Returns |
|---|---|
| `compile(doc, selector)` | `Result<Selector, Error>` |
| `find(doc)` | `Vec<Node>` |
| `find_first(doc)` | `Option<Node>` |
| `find_from(doc, root)` | `Vec<Node>` |
| `find_first_from(doc, root)` | `Option<Node>` |
| `matches(doc, node)` | `bool` |

### TextOpts

```rust
pub struct TextOpts {
    pub deep: bool,           // default: true
    pub separator: &'static str,  // default: ""
    pub strip: bool,          // default: false
    pub skip_empty: bool,     // default: false
}
```

### SerializeOpts

```rust
pub struct SerializeOpts {
    pub indent: usize,            // default: 0
    pub skip_ws_nodes: bool,
    pub skip_comment: bool,
    pub raw: bool,
    pub without_closing: bool,
    pub tag_with_ns: bool,
    pub without_text_indent: bool,
    pub full_doctype: bool,
    pub html5test: bool,
}
```

## Safety

All `unsafe` blocks are isolated and documented with their safety invariants:

- **FFI callbacks** are wrapped in `std::panic::catch_unwind`, aborting on panic to prevent UB
- **`HtmlDocument`** is `!Send` and `!Sync` — lexbor uses per-document memory arenas
- **Drop ordering** is deterministic: selectors → CSS parser → HTML document
- **Node lifetimes** are tied to their owning document via `PhantomData<&'a HtmlDocument>`

## Performance

The library is a zero-overhead wrapper — every function call translates directly
to the underlying lexbor C API. Key optimizations:

- **LTO** and `codegen-units = 1` in release profile
- **Pre-compiled selectors** avoid re-parsing for repeated queries
- **`tag_id()`** provides integer tag comparisons (no string hashing)
- **`select_first()`** short-circuits after the first match

Run benchmarks:

```bash
cargo bench --features vendor
```

## Requirements

- Rust 1.82+ (edition 2024)
- lexbor C library (auto-built with `vendor` feature)
- `git`, `cmake`, C compiler (when using `vendor` feature)

## License

MIT
