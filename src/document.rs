//! HTML document parser and CSS selector engine.
//!
//! `HtmlDocument` owns a parsed HTML tree and provides CSS selector matching.
//! It is **not** `Send` or `Sync` — lexbor is single-threaded.

use std::marker::PhantomData;
use std::ptr;

use crate::callbacks::collect_nodes_callback;
use crate::error::Error;
use crate::ffi::{
    lxb_css_parser_create, lxb_css_parser_destroy, lxb_css_parser_init, lxb_css_parser_t,
    lxb_css_selectors_parse, lxb_html_document_create, lxb_html_document_destroy,
    lxb_html_document_parse, lxb_html_document_parse_fragment, lxb_html_document_t,
    lxb_selectors_create, lxb_selectors_destroy, lxb_selectors_find, lxb_selectors_init,
    lxb_selectors_t, lexbor_status_t_LXB_STATUS_OK,
};
use crate::node::Node;

// ---------------------------------------------------------------------------
// HtmlDocument
// ---------------------------------------------------------------------------

/// A parsed HTML document that supports CSS selector queries.
///
/// # Thread safety
///
/// This type is `!Send` and `!Sync`. Lexbor uses per-document arenas and is not
/// thread-safe. Keep documents on the thread that created them.
pub struct HtmlDocument {
    pub(crate) document: *mut lxb_html_document_t,
    pub(crate) css_parser: *mut lxb_css_parser_t,
    pub(crate) selectors: *mut lxb_selectors_t,
    // Prevent Send + Sync (lexbor is single-threaded).
    _not_send: PhantomData<*mut ()>,
}

impl HtmlDocument {
    /// Parse an HTML string and create a new document.
    pub fn parse(html: &str) -> Result<Self, Error> {
        unsafe {
            let document = lxb_html_document_create();
            if document.is_null() {
                return Err(Error::DocumentCreate);
            }

            let html_bytes = html.as_bytes();
            let status =
                lxb_html_document_parse(document, html_bytes.as_ptr(), html_bytes.len());
            if status != lexbor_status_t_LXB_STATUS_OK {
                lxb_html_document_destroy(document);
                return Err(Error::ParseHtml);
            }

            Self::init_selectors(document)
        }
    }

    /// Internal: create CSS parser and selectors engine.
    unsafe fn init_selectors(document: *mut lxb_html_document_t) -> Result<Self, Error> {
        let css_parser = unsafe { lxb_css_parser_create() };
        if css_parser.is_null() {
            unsafe { lxb_html_document_destroy(document) };
            return Err(Error::CssParserCreate);
        }
        let status = unsafe { lxb_css_parser_init(css_parser, ptr::null_mut()) };
        if status != lexbor_status_t_LXB_STATUS_OK {
            unsafe {
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
            }
            return Err(Error::CssParserCreate);
        }

        let selectors = unsafe { lxb_selectors_create() };
        if selectors.is_null() {
            unsafe {
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
            }
            return Err(Error::SelectorsCreate);
        }
        let status = unsafe { lxb_selectors_init(selectors) };
        if status != lexbor_status_t_LXB_STATUS_OK {
            unsafe {
                lxb_selectors_destroy(selectors, true);
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
            }
            return Err(Error::SelectorsCreate);
        }

        Ok(HtmlDocument {
            document,
            css_parser,
            selectors,
            _not_send: PhantomData,
        })
    }

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

    /// Returns the document's root node.
    pub fn root(&self) -> Node<'_> {
        unsafe {
            let root = crate::ffi::lxb_dom_document_root(&mut (*self.document).dom_document);
            Node {
                node: root as *mut crate::ffi::lxb_dom_node_t,
                _marker: PhantomData,
            }
        }
    }

    /// Returns the document's `<head>` element, if it exists.
    pub fn head(&self) -> Option<Node<'_>> {
        unsafe {
            let head = crate::ffi::lxb_html_document_head_element_noi(self.document);
            if head.is_null() {
                None
            } else {
                Some(Node {
                    node: head as *mut crate::ffi::lxb_dom_node_t,
                    _marker: PhantomData,
                })
            }
        }
    }

    /// Returns the document's `<body>` element, if it exists.
    pub fn body(&self) -> Option<Node<'_>> {
        unsafe {
            let body = crate::ffi::lxb_html_document_body_element_noi(self.document);
            if body.is_null() {
                None
            } else {
                Some(Node {
                    node: body as *mut crate::ffi::lxb_dom_node_t,
                    _marker: PhantomData,
                })
            }
        }
    }

    /// Returns the body node (or document root as fallback) for selector scoping.
    pub(crate) fn body_node(&self) -> Node<'_> {
        self.body().unwrap_or_else(|| self.root())
    }

    // -----------------------------------------------------------------------
    // CSS Selection
    // -----------------------------------------------------------------------

    /// Select all nodes matching a CSS selector, scoped to the document body.
    pub fn select<'a>(&'a self, selector: &str) -> Vec<Node<'a>> {
        let body = self.body_node();
        self.select_from(&body, selector)
    }

    /// Select all nodes matching a CSS selector, scoped to the given root.
    pub fn select_from<'a>(&'a self, root: &Node<'a>, selector: &str) -> Vec<Node<'a>> {
        unsafe {
            let selector_bytes = selector.as_bytes();
            let list = lxb_css_selectors_parse(
                self.css_parser,
                selector_bytes.as_ptr(),
                selector_bytes.len(),
            );
            if list.is_null() {
                return Vec::new();
            }

            let mut nodes = Vec::new();
            lxb_selectors_find(
                self.selectors,
                root.node,
                list,
                Some(collect_nodes_callback),
                &mut nodes as *mut _ as *mut std::os::raw::c_void,
            );
            nodes
        }
    }

    /// Select the first node matching a CSS selector.
    pub fn select_first<'a>(&'a self, selector: &str) -> Option<Node<'a>> {
        let body = self.body_node();
        unsafe {
            let selector_bytes = selector.as_bytes();
            let list = lxb_css_selectors_parse(
                self.css_parser,
                selector_bytes.as_ptr(),
                selector_bytes.len(),
            );
            if list.is_null() {
                return None;
            }

            let mut result: Option<Node> = None;
            lxb_selectors_find(
                self.selectors,
                body.node,
                list,
                Some(find_first_callback),
                &mut result as *mut _ as *mut std::os::raw::c_void,
            );
            result
        }
    }

    /// Find all elements with a given tag name (uses CSS selection internally).
    pub fn tags(&self, name: &str) -> Vec<Node<'_>> {
        self.select(name)
    }

    /// Check if any node in the document matches a CSS selector.
    pub fn matches(&self, selector: &str) -> bool {
        self.select_first(selector).is_some()
    }

    // -----------------------------------------------------------------------
    // Document-level mutation
    // -----------------------------------------------------------------------

    /// Remove all elements with the given tag names from the document.
    pub fn strip_tags(&self, tags: &[&str]) {
        for tag in tags {
            for node in self.tags(tag) {
                let _ = node.decompose();
            }
        }
    }

    // -----------------------------------------------------------------------
    // Pre-compiled selector factory
    // -----------------------------------------------------------------------

    /// Compile a CSS selector for repeated use.
    pub fn compile_selector(
        &self,
        selector: &str,
    ) -> Result<crate::selector::Selector<'_>, Error> {
        crate::selector::Selector::compile(self, selector)
    }
}

impl Drop for HtmlDocument {
    fn drop(&mut self) {
        unsafe {
            lxb_selectors_destroy(self.selectors, true);
            lxb_css_parser_destroy(self.css_parser, true);
            lxb_html_document_destroy(self.document);
        }
    }
}

// ---------------------------------------------------------------------------
// Internal: find-first callback
// ---------------------------------------------------------------------------

unsafe extern "C" fn find_first_callback_impl(
    node: *mut crate::ffi::lxb_dom_node_t,
    _spec: crate::ffi::lxb_css_selector_specificity_t,
    ctx: *mut std::os::raw::c_void,
) -> crate::ffi::lxb_status_t {
    let result = unsafe { &mut *(ctx as *mut Option<Node>) };
    if result.is_none() {
        *result = Some(Node {
            node,
            _marker: PhantomData,
        });
    }
    // Return non-zero to stop iterating after first match.
    1
}

pub(crate) unsafe extern "C" fn find_first_callback(
    node: *mut crate::ffi::lxb_dom_node_t,
    spec: crate::ffi::lxb_css_selector_specificity_t,
    ctx: *mut std::os::raw::c_void,
) -> crate::ffi::lxb_status_t {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        find_first_callback_impl(node, spec, ctx)
    }))
    .unwrap_or_else(|_| std::process::abort())
}
