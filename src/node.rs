//! DOM node wrapper and iterators.
//!
//! Every `Node<'a>` is tied to the lifetime of its owning [`HtmlDocument`](super::HtmlDocument).
//! Nodes must never outlive the document — this is enforced at compile time by the
//! `PhantomData<&'a HtmlDocument>` marker.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::str;

use crate::ffi::{
    lxb_char_t, lxb_dom_attr_t, lxb_dom_element_t, lxb_dom_node_t,
    lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_COMMENT, lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_DOCUMENT,
    lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_ELEMENT, lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_TEXT,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_FULL_DOCTYPE,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_HTML5TEST,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_RAW,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_SKIP_COMMENT,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_SKIP_WS_NODES,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_TAG_WITH_NS,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_WITHOUT_CLOSING,
    lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_WITHOUT_TEXT_INDENT,
    lxb_html_serialize_opt_t, lexbor_status_t_LXB_STATUS_OK,
};

use crate::callbacks::{serialize_callback, serialize_pretty_callback};
use crate::document::HtmlDocument;
use crate::error::Error;

// ---------------------------------------------------------------------------
// SerializeOpts
// ---------------------------------------------------------------------------

/// Options controlling HTML serialization (pretty-printing).
#[derive(Debug, Clone, Copy, Default)]
pub struct SerializeOpts {
    /// Initial indentation level. Default 0.
    pub indent: usize,
    /// Skip text nodes containing only whitespace.
    pub skip_ws_nodes: bool,
    /// Exclude comment nodes from output.
    pub skip_comment: bool,
    /// Serialize text/attribute values without HTML escaping.
    pub raw: bool,
    /// Omit closing tags for non-void elements.
    pub without_closing: bool,
    /// Include namespace prefixes in serialized tag names.
    pub tag_with_ns: bool,
    /// Disable extra indentation added around text/comment content.
    pub without_text_indent: bool,
    /// Serialize full document type declaration.
    pub full_doctype: bool,
    /// Use lexbor's HTML5 test formatting mode.
    pub html5test: bool,
}

impl SerializeOpts {
    fn to_c_flags(&self) -> lxb_html_serialize_opt_t {
        let mut flags: u32 = 0;
        if self.skip_ws_nodes {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_SKIP_WS_NODES;
        }
        if self.skip_comment {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_SKIP_COMMENT;
        }
        if self.raw {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_RAW;
        }
        if self.without_closing {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_WITHOUT_CLOSING;
        }
        if self.tag_with_ns {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_TAG_WITH_NS;
        }
        if self.without_text_indent {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_WITHOUT_TEXT_INDENT;
        }
        if self.full_doctype {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_FULL_DOCTYPE;
        }
        if self.html5test {
            flags |= lxb_html_serialize_opt_LXB_HTML_SERIALIZE_OPT_HTML5TEST;
        }
        flags as lxb_html_serialize_opt_t
    }
}

// ---------------------------------------------------------------------------
// TextOpts
// ---------------------------------------------------------------------------

/// Options controlling text extraction.
#[derive(Debug, Clone, Copy)]
pub struct TextOpts {
    /// Include text from all descendants (true) or only direct children (false).
    pub deep: bool,
    /// String inserted between successive text fragments.
    pub separator: &'static str,
    /// Apply `str::trim()` to each fragment before joining.
    pub strip: bool,
    /// Exclude text nodes that contain only ASCII whitespace.
    pub skip_empty: bool,
}

impl Default for TextOpts {
    fn default() -> Self {
        TextOpts {
            deep: true,
            separator: "",
            strip: false,
            skip_empty: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A reference to a DOM node inside an HTML document.
///
/// The lifetime `'a` is tied to the [`HtmlDocument`] that owns the node.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    pub(crate) node: *mut lxb_dom_node_t,
    pub(crate) _marker: PhantomData<&'a HtmlDocument>,
}

// ---------------------------------------------------------------------------
// Traversal
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Returns the parent node, if any.
    pub fn parent(&self) -> Option<Node<'a>> {
        let p = unsafe { crate::ffi::lxb_dom_node_parent_noi(self.node) };
        ptr_to_node(p)
    }

    /// Returns the first child node, if any.
    pub fn first_child(&self) -> Option<Node<'a>> {
        let c = unsafe { crate::ffi::lxb_dom_node_first_child_noi(self.node) };
        ptr_to_node(c)
    }

    /// Returns the last child node, if any.
    pub fn last_child(&self) -> Option<Node<'a>> {
        let c = unsafe { crate::ffi::lxb_dom_node_last_child_noi(self.node) };
        ptr_to_node(c)
    }

    /// Returns the next sibling, if any.
    pub fn next_sibling(&self) -> Option<Node<'a>> {
        let s = unsafe { crate::ffi::lxb_dom_node_next_noi(self.node) };
        ptr_to_node(s)
    }

    /// Returns the previous sibling, if any.
    pub fn prev_sibling(&self) -> Option<Node<'a>> {
        let s = unsafe { crate::ffi::lxb_dom_node_prev_noi(self.node) };
        ptr_to_node(s)
    }

    /// Returns an iterator over this node's direct children.
    pub fn children(&self) -> ChildrenIter<'a> {
        ChildrenIter {
            current: self.first_child(),
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over all descendants (depth-first).
    pub fn descendants(&self) -> DescendantsIter<'a> {
        DescendantsIter {
            root: *self,
            current: Some(*self),
            _marker: PhantomData,
        }
    }

    /// Returns an iterator walking up the tree from this node to the root.
    pub fn ancestors(&self) -> AncestorsIter<'a> {
        AncestorsIter {
            current: Some(*self),
        }
    }

    /// Depth-first traversal starting from this node, yielding this node first.
    pub fn traverse(&self) -> TraverseIter<'a> {
        TraverseIter {
            root: *self,
            current: Some(*self),
            started: false,
            _marker: PhantomData,
        }
    }

    /// Iterate over direct children, optionally including text nodes.
    pub fn iter_children(
        &self,
        include_text: bool,
        skip_empty: bool,
    ) -> impl Iterator<Item = Node<'a>> + '_ {
        self.children().filter(move |n| {
            if n.is_element() {
                return true;
            }
            if include_text && n.is_text() {
                if skip_empty && n.is_empty_text_node() {
                    return false;
                }
                return true;
            }
            false
        })
    }
}

// ---------------------------------------------------------------------------
// Type checks
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Returns the raw node type constant from lexbor.
    pub fn node_type(&self) -> u32 {
        unsafe { (*self.node).type_ }
    }

    /// `true` for element nodes.
    pub fn is_element(&self) -> bool {
        self.node_type() == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_ELEMENT
    }

    /// `true` for text nodes.
    pub fn is_text(&self) -> bool {
        self.node_type() == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_TEXT
    }

    /// `true` for comment nodes.
    pub fn is_comment(&self) -> bool {
        self.node_type() == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_COMMENT
    }

    /// `true` for document nodes.
    pub fn is_document(&self) -> bool {
        self.node_type() == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_DOCUMENT
    }

    /// `true` if this is a text node containing only ASCII whitespace.
    pub fn is_empty_text_node(&self) -> bool {
        if !self.is_text() {
            return false;
        }
        self.text_content()
            .map(|t| {
                t.as_bytes()
                    .iter()
                    .all(|&b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0x0c)
            })
            .unwrap_or(true)
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Returns the lexbor internal tag ID (integer) for fast comparisons.
    pub fn tag_id(&self) -> usize {
        unsafe { crate::ffi::lxb_dom_node_tag_id_noi(self.node) as usize }
    }

    /// Returns the tag name of an element node (e.g. `"div"`, `"p"`).
    /// Returns `""` for non-element nodes.
    pub fn tag_name(&self) -> String {
        if !self.is_element() {
            return String::new();
        }
        let mut len: usize = 0;
        let ptr = unsafe {
            crate::ffi::lxb_dom_element_tag_name(self.node as *mut lxb_dom_element_t, &mut len)
        };
        if ptr.is_null() || len == 0 {
            return String::new();
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        str::from_utf8(slice).unwrap_or("").to_ascii_lowercase()
    }

    /// Returns the `id` attribute value, if set.
    pub fn id(&self) -> Option<&str> {
        self.attr("id")
    }

    /// Returns the `class` attribute value, if set.
    pub fn class(&self) -> Option<&str> {
        self.attr("class")
    }

    /// Returns the value of an attribute by name, or `None`.
    pub fn attr(&self, name: &str) -> Option<&str> {
        if !self.is_element() {
            return None;
        }
        let element = self.node as *mut lxb_dom_element_t;
        let name_c = std::ffi::CString::new(name).ok()?;
        let mut value_len: usize = 0;
        let value_ptr = unsafe {
            crate::ffi::lxb_dom_element_get_attribute(
                element,
                name_c.as_ptr() as *const lxb_char_t,
                name.len(),
                &mut value_len,
            )
        };
        if value_ptr.is_null() {
            return None;
        }
        let slice = unsafe { std::slice::from_raw_parts(value_ptr, value_len) };
        str::from_utf8(slice).ok()
    }

    /// Returns true if the node has an attribute with the given name.
    pub fn has_attr(&self, name: &str) -> bool {
        self.attr(name).is_some()
    }

    /// Returns true if this node is an element with the given class.
    pub fn has_class(&self, name: &str) -> bool {
        self.class()
            .map(|c| c.split_ascii_whitespace().any(|s| s == name))
            .unwrap_or(false)
    }

    /// Returns all class names as a `Vec` of `&str`.
    pub fn classes(&self) -> Vec<&str> {
        self.class()
            .map(|c| c.split_ascii_whitespace().collect())
            .unwrap_or_default()
    }

    /// Returns all attributes as a `HashMap<&str, &str>`.
    pub fn attributes(&self) -> HashMap<&str, &str> {
        let mut attrs = HashMap::new();
        if !self.is_element() {
            return attrs;
        }
        let element = self.node as *mut lxb_dom_element_t;
        let mut attr: *mut lxb_dom_attr_t =
            unsafe { crate::ffi::lxb_dom_element_first_attribute_noi(element) };
        while !attr.is_null() {
            let mut name_len: usize = 0;
            let name_ptr = unsafe { crate::ffi::lxb_dom_attr_qualified_name(attr, &mut name_len) };
            let mut value_len: usize = 0;
            let value_ptr = unsafe { crate::ffi::lxb_dom_attr_value_noi(attr, &mut value_len) };
            if !name_ptr.is_null() && !value_ptr.is_null() {
                let name_slice = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
                let value_slice = unsafe { std::slice::from_raw_parts(value_ptr, value_len) };
                if let (Ok(name), Ok(value)) =
                    (str::from_utf8(name_slice), str::from_utf8(value_slice))
                {
                    attrs.insert(name, value);
                }
            }
            attr = unsafe { crate::ffi::lxb_dom_element_next_attribute_noi(attr) };
        }
        attrs
    }
}

// ---------------------------------------------------------------------------
// Text content
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Returns the text content of this specific node (no children).
    /// Only returns data for text and comment nodes; returns `None` for elements.
    pub fn text_content(&self) -> Option<&str> {
        if !self.is_text() && !self.is_comment() {
            return None;
        }
        let char_data = self.node as *mut crate::ffi::lxb_dom_character_data_t;
        let data_ptr =
            unsafe { crate::ffi::lexbor_str_data_noi(&mut (*char_data).data) };
        if data_ptr.is_null() {
            return None;
        }
        let len = unsafe { (*char_data).data.length };
        if len == 0 {
            return Some("");
        }
        let slice = unsafe { std::slice::from_raw_parts(data_ptr, len) };
        str::from_utf8(slice).ok()
    }

    /// Returns the comment content (text inside `<!-- -->`), or `None`.
    pub fn comment_content(&self) -> Option<&str> {
        if !self.is_comment() {
            return None;
        }
        self.text_content()
    }

    /// Returns the text content of this node (all descendant text concatenated).
    pub fn text(&self) -> String {
        self.text_with(TextOpts::default())
    }

    /// Returns text content with configurable options.
    pub fn text_with(&self, opts: TextOpts) -> String {
        if opts.deep {
            self.text_deep(opts)
        } else {
            self.text_shallow(opts)
        }
    }

    fn text_deep(&self, opts: TextOpts) -> String {
        let mut len: usize = 0;
        let ptr = unsafe { crate::ffi::lxb_dom_node_text_content(self.node, &mut len) };
        if ptr.is_null() || len == 0 {
            return String::new();
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        let s = String::from_utf8_lossy(slice);
        if opts.strip {
            s.trim().to_string()
        } else {
            s.into_owned()
        }
    }

    fn text_shallow(&self, opts: TextOpts) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Self, if text
        if self.is_text() {
            if let Some(t) = self.text_content() {
                if !opts.skip_empty || !self.is_empty_text_node() {
                    let t = if opts.strip { t.trim() } else { t };
                    if !t.is_empty() || !opts.skip_empty {
                        parts.push(t.to_string());
                    }
                }
            }
        }

        // Direct children that are text nodes
        let mut child_ptr = unsafe { (*self.node).first_child };
        while !child_ptr.is_null() {
            let child_type = unsafe { (*child_ptr).type_ };
            if child_type == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_TEXT {
                let char_data = child_ptr as *mut crate::ffi::lxb_dom_character_data_t;
                let data_ptr =
                    unsafe { crate::ffi::lexbor_str_data_noi(&mut (*char_data).data) };
                if !data_ptr.is_null() {
                    let data_len = unsafe { (*char_data).data.length };
                    if data_len > 0 {
                        let slice = unsafe { std::slice::from_raw_parts(data_ptr, data_len) };
                        if let Ok(s) = str::from_utf8(slice) {
                            let is_empty = s.as_bytes().iter().all(|&b| {
                                b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' || b == 0x0c
                            });
                            if !opts.skip_empty || !is_empty {
                                let s = if opts.strip { s.trim() } else { s };
                                if !s.is_empty() || !opts.skip_empty {
                                    parts.push(s.to_string());
                                }
                            }
                        }
                    }
                }
            }
            child_ptr = unsafe { (*child_ptr).next };
        }

        parts.join(opts.separator)
    }
}

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Returns the inner HTML of this node (serialised children, not the node itself).
    pub fn inner_html(&self) -> String {
        let mut result = String::new();
        let mut child = unsafe { (*self.node).first_child };
        while !child.is_null() {
            unsafe {
                crate::ffi::lxb_html_serialize_tree_cb(
                    child,
                    Some(serialize_callback),
                    &mut result as *mut String as *mut std::os::raw::c_void,
                );
            }
            child = unsafe { (*child).next };
        }
        result
    }

    /// Returns the outer HTML of this node (the node itself and its descendants).
    pub fn outer_html(&self) -> String {
        let mut result = String::new();
        unsafe {
            crate::ffi::lxb_html_serialize_tree_cb(
                self.node,
                Some(serialize_callback),
                &mut result as *mut String as *mut std::os::raw::c_void,
            );
        }
        result
    }

    /// Pretty-printed outer HTML with serialization options.
    pub fn outer_html_pretty(&self, opts: SerializeOpts) -> String {
        let mut result = String::new();
        let flags = opts.to_c_flags();
        unsafe {
            crate::ffi::lxb_html_serialize_pretty_tree_cb(
                self.node,
                flags,
                opts.indent,
                Some(serialize_pretty_callback),
                &mut result as *mut String as *mut std::os::raw::c_void,
            );
        }
        result
    }

    /// Pretty-printed inner HTML with serialization options.
    pub fn inner_html_pretty(&self, opts: SerializeOpts) -> String {
        let mut result = String::new();
        let flags = opts.to_c_flags();
        unsafe {
            crate::ffi::lxb_html_serialize_pretty_deep_cb(
                self.node,
                flags,
                opts.indent,
                Some(serialize_pretty_callback),
                &mut result as *mut String as *mut std::os::raw::c_void,
            );
        }
        result
    }
}

// ---------------------------------------------------------------------------
// DOM Mutation
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Remove this node from the tree (shallow — does not remove children).
    /// Children are re-parented to this node's parent.
    /// Returns `Err` if this is the document root.
    pub fn decompose_shallow(&self) -> Result<(), Error> {
        if self.is_document() {
            return Err(Error::RootNode);
        }
        unsafe {
            crate::ffi::lxb_dom_node_remove(self.node);
        }
        Ok(())
    }

    /// Remove this node and all its children from the tree.
    /// Returns `Err` if this is the document root.
    pub fn decompose(&self) -> Result<(), Error> {
        if self.is_document() {
            return Err(Error::RootNode);
        }
        node_remove_deep(self.node);
        Ok(())
    }

    /// Alias for `decompose()`.
    pub fn remove(&self) -> Result<(), Error> {
        self.decompose()
    }

    /// Unwrap this node: move its children to the parent, then remove this node.
    pub fn unwrap(&self) -> Result<(), Error> {
        if self.is_document() {
            return Err(Error::RootNode);
        }
        // Collect all children first (avoid modifying while iterating)
        let children: Vec<Node<'a>> = self.children().collect();
        for child in &children {
            unsafe {
                crate::ffi::lxb_dom_node_remove(child.node);
                crate::ffi::lxb_dom_node_insert_before_wo_events(child.node, self.node);
            }
        }
        self.decompose()
    }

    /// Append a text node as a child of this node.
    ///
    /// Uses `lxb_dom_node_append_child` which properly integrates the node
    /// into the DOM tree (unlike the `_wo_events` variants).
    pub fn append_text(&self, text: &str) -> Result<(), Error> {
        unsafe {
            let owner_doc = (*self.node).owner_document;
            let text_node_ptr = crate::ffi::lxb_dom_document_create_text_node(
                owner_doc,
                text.as_ptr() as *const lxb_char_t,
                text.len(),
            );
            if text_node_ptr.is_null() {
                return Err(Error::InsertFragment("failed to create text node".into()));
            }
            crate::ffi::lxb_dom_node_append_child(
                self.node,
                text_node_ptr as *mut lxb_dom_node_t,
            );
        }
        Ok(())
    }

    /// Merge adjacent text nodes among this node's descendants.
    /// Walks the subtree merging adjacent text nodes into a single node.
    pub fn merge_text_nodes(&self) {
        unsafe fn is_text(node: *mut lxb_dom_node_t) -> bool {
            !node.is_null()
                && unsafe { (*node).type_ == lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_TEXT }
        }

        let mut child_ptr = unsafe { (*self.node).first_child };
        while !child_ptr.is_null() {
            if unsafe { is_text(child_ptr) } {
                let next_ptr = unsafe { (*child_ptr).next };
                if unsafe { is_text(next_ptr) } {
                    // Get text of next, append to current, remove next
                    let mut next_len: usize = 0;
                    let next_text =
                        unsafe { crate::ffi::lxb_dom_node_text_content(next_ptr, &mut next_len) };
                    if !next_text.is_null() && next_len > 0 {
                        let mut cur_len: usize = 0;
                        let cur_text =
                            unsafe { crate::ffi::lxb_dom_node_text_content(child_ptr, &mut cur_len) };
                        let total = cur_len + next_len;
                        let mut merged: Vec<u8> = Vec::with_capacity(total);
                        if !cur_text.is_null() && cur_len > 0 {
                            merged.extend_from_slice(unsafe {
                                std::slice::from_raw_parts(cur_text, cur_len)
                            });
                        }
                        merged.extend_from_slice(unsafe {
                            std::slice::from_raw_parts(next_text, next_len)
                        });
                        unsafe {
                            crate::ffi::lxb_dom_node_text_content_set(
                                child_ptr,
                                merged.as_ptr() as *const lxb_char_t,
                                merged.len(),
                            );
                        }
                    }
                    unsafe {
                        crate::ffi::lxb_dom_node_remove(next_ptr);
                    }
                    continue;
                }
            }
            child_ptr = unsafe { (*child_ptr).next };
        }
    }
}

// ---------------------------------------------------------------------------
// Attribute Mutation
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Set an attribute value. If `value` is `None`, the attribute is created
    /// as an empty attribute (boolean attribute style).
    pub fn set_attr(&self, name: &str, value: Option<&str>) -> Result<(), Error> {
        if !self.is_element() {
            return Err(Error::SetAttribute("not an element node".into()));
        }
        let element = self.node as *mut lxb_dom_element_t;
        let name_c = std::ffi::CString::new(name)
            .map_err(|_| Error::SetAttribute(format!("invalid name: {name}")))?;
        let (val_ptr, val_len) = match value {
            Some(v) => (v.as_ptr() as *const lxb_char_t, v.len()),
            None => (std::ptr::null(), 0),
        };
        let attr = unsafe {
            crate::ffi::lxb_dom_element_set_attribute(
                element,
                name_c.as_ptr() as *const lxb_char_t,
                name.len(),
                val_ptr,
                val_len,
            )
        };
        if attr.is_null() {
            return Err(Error::SetAttribute(name.to_string()));
        }
        Ok(())
    }

    /// Remove an attribute by name.
    pub fn remove_attr(&self, name: &str) -> Result<(), Error> {
        if !self.is_element() {
            return Err(Error::RemoveAttribute("not an element node".into()));
        }
        let element = self.node as *mut lxb_dom_element_t;
        let name_c = std::ffi::CString::new(name)
            .map_err(|_| Error::RemoveAttribute(format!("invalid name: {name}")))?;
        unsafe {
            crate::ffi::lxb_dom_element_remove_attribute(
                element,
                name_c.as_ptr() as *const lxb_char_t,
                name.len(),
            );
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Selector matching on nodes
// ---------------------------------------------------------------------------

impl<'a> Node<'a> {
    /// Check if this specific node matches a CSS selector.
    ///
    /// This does NOT search children — it only tests this node.
    /// Creates a temporary CSS parser, so for repeated use prefer
    /// [`Selector::matches`](super::Selector::matches).
    pub fn css_matches(&self, selector: &str) -> bool {
        unsafe {
            let parser = crate::ffi::lxb_css_parser_create();
            if parser.is_null() {
                return false;
            }
            let status = crate::ffi::lxb_css_parser_init(parser, std::ptr::null_mut());
            if status != lexbor_status_t_LXB_STATUS_OK {
                crate::ffi::lxb_css_parser_destroy(parser, true);
                return false;
            }

            let selectors = crate::ffi::lxb_selectors_create();
            if selectors.is_null() {
                crate::ffi::lxb_css_parser_destroy(parser, true);
                return false;
            }
            let status = crate::ffi::lxb_selectors_init(selectors);
            if status != lexbor_status_t_LXB_STATUS_OK {
                crate::ffi::lxb_selectors_destroy(selectors, true);
                crate::ffi::lxb_css_parser_destroy(parser, true);
                return false;
            }

            let bytes = selector.as_bytes();
            let list =
                crate::ffi::lxb_css_selectors_parse(parser, bytes.as_ptr(), bytes.len());
            if list.is_null() {
                crate::ffi::lxb_selectors_destroy(selectors, true);
                crate::ffi::lxb_css_parser_destroy(parser, true);
                return false;
            }

            let mut matched = false;
            unsafe extern "C" fn match_cb(
                _node: *mut crate::ffi::lxb_dom_node_t,
                _spec: crate::ffi::lxb_css_selector_specificity_t,
                ctx: *mut std::os::raw::c_void,
            ) -> crate::ffi::lxb_status_t {
                let m = unsafe { &mut *(ctx as *mut bool) };
                *m = true;
                1
            }

            crate::ffi::lxb_selectors_match_node(
                selectors,
                self.node,
                list,
                Some(match_cb),
                &mut matched as *mut _ as *mut std::os::raw::c_void,
            );

            crate::ffi::lxb_css_selector_list_destroy_memory(list);
            crate::ffi::lxb_selectors_destroy(selectors, true);
            crate::ffi::lxb_css_parser_destroy(parser, true);
            matched
        }
    }

    /// Check if any of the given CSS selectors match this node.
    pub fn any_css_matches(&self, selectors: &[&str]) -> bool {
        selectors.iter().any(|s| self.css_matches(*s))
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Remove a node and all its descendants (iterative).
fn node_remove_deep(root: *mut lxb_dom_node_t) {
    let mut node = root;
    loop {
        if node.is_null() {
            break;
        }
        let first_child = unsafe { (*node).first_child };
        if !first_child.is_null() {
            node = first_child;
        } else {
            while node != root && unsafe { (*node).next }.is_null() {
                let parent = unsafe { (*node).parent };
                unsafe {
                    crate::ffi::lxb_dom_node_remove(node);
                }
                node = parent;
            }

            if node == root {
                unsafe {
                    crate::ffi::lxb_dom_node_remove(node);
                }
                break;
            }

            let next_sib = unsafe { (*node).next };
            unsafe {
                crate::ffi::lxb_dom_node_remove(node);
            }
            node = next_sib;
        }
    }
}

fn ptr_to_node<'a>(ptr: *mut lxb_dom_node_t) -> Option<Node<'a>> {
    if ptr.is_null() {
        None
    } else {
        Some(Node {
            node: ptr,
            _marker: PhantomData,
        })
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Iterator over the direct children of a node.
pub struct ChildrenIter<'a> {
    current: Option<Node<'a>>,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> Iterator for ChildrenIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.next_sibling();
        Some(node)
    }
}

/// Depth-first iterator over all descendants of a node.
pub struct DescendantsIter<'a> {
    root: Node<'a>,
    current: Option<Node<'a>>,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> Iterator for DescendantsIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        if let Some(child) = current.first_child() {
            self.current = Some(child);
            return Some(child);
        }
        let mut node = current;
        loop {
            if node.node == self.root.node {
                self.current = None;
                return None;
            }
            if let Some(sib) = node.next_sibling() {
                self.current = Some(sib);
                return Some(sib);
            }
            node = node.parent()?;
        }
    }
}

/// Iterator walking up from a node to the root (parent chain).
pub struct AncestorsIter<'a> {
    current: Option<Node<'a>>,
}

impl<'a> Iterator for AncestorsIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.parent();
        Some(node)
    }
}

/// Depth-first traverse starting from and including the root node.
pub struct TraverseIter<'a> {
    root: Node<'a>,
    current: Option<Node<'a>>,
    started: bool,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> Iterator for TraverseIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            return Some(self.root);
        }
        let current = self.current?;
        if let Some(child) = current.first_child() {
            self.current = Some(child);
            return Some(child);
        }
        let mut node = current;
        loop {
            if node.node == self.root.node {
                self.current = None;
                return None;
            }
            if let Some(sib) = node.next_sibling() {
                self.current = Some(sib);
                return Some(sib);
            }
            node = node.parent()?;
        }
    }
}
