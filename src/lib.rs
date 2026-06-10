//! # lexbor_css — Safe, fast CSS selection for Rust
//!
//! A Rust library that wraps the [lexbor](https://github.com/lexbor/lexbor) C
//! HTML parser and CSS selector engine, providing a safe, ergonomic API
//! comparable to Python's [selectolax](https://github.com/rushter/selectolax).
//!
//! ## Features
//!
//! - **Safe**: All `unsafe` blocks are isolated and documented with safety
//!   invariants. Panics in FFI callbacks are caught to prevent UB.
//! - **Fast**: Thin wrapper over lexbor's C engine; pre-compiled selectors;
//!   minimal allocations on the hot path.
//! - **CSS3 selectors**: Full support for tag, class, id, attribute, pseudo-class,
//!   and combinator selectors (powered by lexbor).
//! - **DOM traversal**: Parent, children, siblings, descendants, ancestors, traverse.
//! - **DOM mutation**: Decompose, unwrap, replace, insert, append, merge text nodes.
//! - **Fragment parsing**: Parse HTML fragments without `<html>`/`<head>`/`<body>` auto-insertion.
//! - **Pretty printing**: Configurable HTML serialization with 9 options.
//!
//! ## Quick start
//!
//! ```ignore
//! use lexbor_css::HtmlDocument;
//!
//! let doc = HtmlDocument::parse("<div class='main'><p>Hello</p></div>")?;
//!
//! // Ad-hoc selection
//! for node in doc.select("div.main p") {
//!     println!("{}", node.text());
//! }
//!
//! // Pre-compiled selector (faster for repeated use)
//! let sel = doc.compile_selector("p")?;
//! for node in sel.find(&doc) {
//!     println!("tag={} text={}", node.tag_name(), node.text());
//! }
//! # Ok::<(), lexbor_css::Error>(())
//! ```
//!
//! ## Thread safety
//!
//! `HtmlDocument` (and all `Node` references derived from it) are `!Send` and
//! `!Sync`. Lexbor uses per-document memory arenas and is not thread-safe.
//! Keep documents on the thread that created them.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unnecessary_transmutes)]
#![allow(clippy::useless_transmute)]
#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_imports)]

// Generated bindings
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod callbacks;
mod collection;
mod css;
mod document;
mod error;
pub mod node;
mod selector;

pub use collection::Collection;
pub use css::StyleSheet;
pub use document::HtmlDocument;
pub use error::Error;
pub use node::{Node, SerializeOpts, TextOpts};
pub use selector::Selector;
