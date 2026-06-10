//! Pre-compiled CSS selector for repeated use.
//!
//! Compile a selector once and reuse it across multiple documents or queries
//! for better performance.

use std::marker::PhantomData;

use crate::callbacks::collect_nodes_callback;
use crate::document::HtmlDocument;
use crate::error::Error;
use crate::ffi::{
    lxb_css_selector_list_destroy, lxb_css_selector_list_t, lxb_css_selectors_parse,
    lxb_selectors_find, lexbor_status_t_LXB_STATUS_OK,
};
use crate::node::Node;

/// A pre-compiled CSS selector.
///
/// The lifetime `'a` is tied to the CSS parser that compiled it (owned by the
/// [`HtmlDocument`] passed to [`Selector::compile`]).
pub struct Selector<'a> {
    list: *mut lxb_css_selector_list_t,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> Selector<'a> {
    /// Compile a CSS selector string.
    pub fn compile(doc: &'a HtmlDocument, selector: &str) -> Result<Self, Error> {
        let list = unsafe {
            let bytes = selector.as_bytes();
            lxb_css_selectors_parse(doc.css_parser, bytes.as_ptr(), bytes.len())
        };
        if list.is_null() {
            return Err(Error::SelectorParse(selector.to_string()));
        }
        Ok(Selector {
            list,
            _marker: PhantomData,
        })
    }

    /// Find all nodes matching this selector, scoped to the document body.
    pub fn find<'b>(&self, doc: &'b HtmlDocument) -> Vec<Node<'b>> {
        let body = doc.body_node();
        self.find_from(doc, &body)
    }

    /// Find all nodes matching this selector, scoped to the given root.
    pub fn find_from<'b>(&self, doc: &'b HtmlDocument, root: &Node<'b>) -> Vec<Node<'b>> {
        let mut nodes = Vec::new();
        unsafe {
            lxb_selectors_find(
                doc.selectors,
                root.node,
                self.list,
                Some(collect_nodes_callback),
                &mut nodes as *mut _ as *mut std::os::raw::c_void,
            );
        }
        nodes
    }

    /// Find the first node matching this selector, scoped to the document body.
    pub fn find_first<'b>(&self, doc: &'b HtmlDocument) -> Option<Node<'b>> {
        let body = doc.body_node();
        self.find_first_from(doc, &body)
    }

    /// Find the first node matching this selector, scoped to the given root.
    pub fn find_first_from<'b>(
        &self,
        doc: &'b HtmlDocument,
        root: &Node<'b>,
    ) -> Option<Node<'b>> {
        let mut result: Option<Node> = None;
        unsafe {
            lxb_selectors_find(
                doc.selectors,
                root.node,
                self.list,
                Some(crate::document::find_first_callback),
                &mut result as *mut _ as *mut std::os::raw::c_void,
            );
        }
        result
    }

    /// Check if a specific node matches this pre-compiled selector.
    ///
    /// This is faster than `node.css_matches()` because the selector is
    /// already compiled.
    pub fn matches<'b>(&self, doc: &'b HtmlDocument, node: &Node<'b>) -> bool {
        // Create a lightweight callback that just records a match
        let mut matched = false;
        unsafe extern "C" fn match_callback(
            _node: *mut crate::ffi::lxb_dom_node_t,
            _spec: crate::ffi::lxb_css_selector_specificity_t,
            ctx: *mut std::os::raw::c_void,
        ) -> crate::ffi::lxb_status_t {
            let matched = unsafe { &mut *(ctx as *mut bool) };
            *matched = true;
            1 // stop after first match
        }
        unsafe {
            crate::ffi::lxb_selectors_match_node(
                doc.selectors,
                node.node,
                self.list,
                Some(match_callback),
                &mut matched as *mut _ as *mut std::os::raw::c_void,
            ) == lexbor_status_t_LXB_STATUS_OK
        }
    }
}

impl<'a> Drop for Selector<'a> {
    fn drop(&mut self) {
        unsafe {
            lxb_css_selector_list_destroy(self.list);
        }
    }
}
