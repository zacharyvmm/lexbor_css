//! FFI callbacks passed to lexbor. Every callback must be `extern "C"` and
//! must NOT unwind into C code (panicking across FFI is undefined behaviour).
//! We wrap each body in `catch_unwind`; if a panic does occur we abort.

use std::os::raw::c_void;
use std::str;

use crate::node::Node;

use crate::ffi::{
    lxb_char_t, lxb_css_selector_specificity_t, lxb_dom_node_t, lxb_status_t,
    lexbor_status_t_LXB_STATUS_OK,
};

// ---------------------------------------------------------------------------
// Collect-all callback — pushes each matched node into a Vec
// ---------------------------------------------------------------------------

unsafe extern "C" fn collect_nodes_callback_impl(
    node: *mut lxb_dom_node_t,
    _spec: lxb_css_selector_specificity_t,
    ctx: *mut c_void,
) -> lxb_status_t {
    let nodes = unsafe { &mut *(ctx as *mut Vec<Node>) };
    nodes.push(Node {
        node,
        _marker: std::marker::PhantomData,
    });
    lexbor_status_t_LXB_STATUS_OK
}

pub(super) unsafe extern "C" fn collect_nodes_callback(
    node: *mut lxb_dom_node_t,
    spec: lxb_css_selector_specificity_t,
    ctx: *mut c_void,
) -> lxb_status_t {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        collect_nodes_callback_impl(node, spec, ctx)
    }))
    .unwrap_or_else(|_| std::process::abort())
}

// ---------------------------------------------------------------------------
// Serialize callback — pushes serialized HTML into a String
// ---------------------------------------------------------------------------

unsafe extern "C" fn serialize_callback_impl(
    data: *const lxb_char_t,
    len: usize,
    ctx: *mut c_void,
) -> lxb_status_t {
    let result = unsafe { &mut *(ctx as *mut String) };
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    if let Ok(s) = str::from_utf8(slice) {
        result.push_str(s);
    }
    lexbor_status_t_LXB_STATUS_OK
}

pub(super) unsafe extern "C" fn serialize_callback(
    data: *const lxb_char_t,
    len: usize,
    ctx: *mut c_void,
) -> lxb_status_t {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        serialize_callback_impl(data, len, ctx)
    }))
    .unwrap_or_else(|_| std::process::abort())
}

/// Callback for pretty-printing serialization. Same as `serialize_callback`
/// but with the signature lexbor expects for pretty-print callbacks.
pub(super) unsafe extern "C" fn serialize_pretty_callback(
    data: *const lxb_char_t,
    len: usize,
    ctx: *mut c_void,
) -> lxb_status_t {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        serialize_callback_impl(data, len, ctx)
    }))
    .unwrap_or_else(|_| std::process::abort())
}
