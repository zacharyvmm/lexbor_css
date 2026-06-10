//! Thin wrappers around lexbor's DOM collection API.
//!
//! DOM collections are live node lists that can be populated by CSS selectors
//! or manual insertion.

use std::marker::PhantomData;

use crate::document::HtmlDocument;
use crate::ffi::{
    lxb_dom_collection_create, lxb_dom_collection_destroy, lxb_dom_collection_t,
};
use crate::node::Node;

/// A live collection of DOM nodes.
///
/// The lifetime `'a` is tied to the owning document.
pub struct Collection<'a> {
    collection: *mut lxb_dom_collection_t,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> Collection<'a> {
    /// Create a new empty collection.
    pub fn new(doc: &'a HtmlDocument) -> Self {
        let collection = unsafe {
            let dom_doc = &mut (*doc.document).dom_document as *mut crate::ffi::lxb_dom_document_t;
            lxb_dom_collection_create(dom_doc)
        };
        Collection {
            collection,
            _marker: PhantomData,
        }
    }

    /// Create a collection from existing nodes.
    pub fn from_nodes(doc: &'a HtmlDocument, nodes: &[Node<'a>]) -> Self {
        let collection = unsafe {
            let dom_doc = &mut (*doc.document).dom_document as *mut crate::ffi::lxb_dom_document_t;
            lxb_dom_collection_create(dom_doc)
        };
        for node in nodes {
            unsafe {
                crate::ffi::lxb_dom_collection_append_noi(
                    collection,
                    node.node as *mut std::os::raw::c_void,
                );
            }
        }
        Collection {
            collection,
            _marker: PhantomData,
        }
    }

    /// Returns the number of nodes in the collection.
    pub fn len(&self) -> usize {
        unsafe { crate::ffi::lxb_dom_collection_length_noi(self.collection) }
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a node by index.
    pub fn get(&self, index: usize) -> Option<Node<'a>> {
        if index >= self.len() {
            return None;
        }
        let node = unsafe { crate::ffi::lxb_dom_collection_node_noi(self.collection, index) };
        if node.is_null() {
            None
        } else {
            Some(Node {
                node,
                _marker: PhantomData,
            })
        }
    }

    /// Returns a pointer to the underlying `lxb_dom_collection_t`.
    pub unsafe fn as_ptr(&self) -> *mut lxb_dom_collection_t {
        self.collection
    }
}

impl<'a> Drop for Collection<'a> {
    fn drop(&mut self) {
        unsafe {
            lxb_dom_collection_destroy(self.collection, true);
        }
    }
}
