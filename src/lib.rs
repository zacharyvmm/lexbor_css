#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::raw::c_void;
use std::ptr;

const LXB_STATUS_OK: u32 = 0;

unsafe extern "C" fn selector_callback(
    node: *mut lxb_dom_node_t,
    _spec: lxb_css_selector_specificity_t,
    ctx: *mut c_void,
) -> lxb_status_t {
    unsafe {
        let count_ptr = ctx as *mut i32;
        *count_ptr += 1;

        let mut len: usize = 0;
        let text_ptr = lxb_dom_node_text_content(node, &mut len);

        if !text_ptr.is_null() {
            let text_slice = std::slice::from_raw_parts(text_ptr, len);
            let text_str = String::from_utf8_lossy(text_slice);

            println!("Match #{}: {}", *count_ptr, text_str);

            //libc::free(text_ptr as *mut c_void);
        } else {
            println!("Match #{}: [No Text Content]", *count_ptr);
        }
    }

    return LXB_STATUS_OK as lxb_status_t;
}

pub fn parse_and_select(html: &str, selector_str: &str) {
    unsafe {
        let html_bytes = html.as_bytes();
        let document = lxb_html_document_create();
        let status = lxb_html_document_parse(document, html_bytes.as_ptr(), html_bytes.len());

        if status != LXB_STATUS_OK as lxb_status_t {
            eprintln!("Failed to parse HTML");
            lxb_html_document_destroy(document);
            return;
        }

        let css_parser = lxb_css_parser_create();
        let status = lxb_css_parser_init(css_parser, ptr::null_mut());

        if status != LXB_STATUS_OK as lxb_status_t {
            eprintln!("Failed to init CSS parser");
            lxb_html_document_destroy(document);
            return;
        }

        let selector_bytes = selector_str.as_bytes();
        let selector_list =
            lxb_css_selectors_parse(css_parser, selector_bytes.as_ptr(), selector_bytes.len());

        if selector_list.is_null() {
            eprintln!("Failed to parse CSS selector string");
            lxb_css_parser_destroy(css_parser, true);
            lxb_html_document_destroy(document);
            return;
        }

        let selectors = lxb_selectors_create();
        let status = lxb_selectors_init(selectors);

        if status != LXB_STATUS_OK as lxb_status_t {
            eprintln!("Failed to init selectors engine");
            lxb_css_parser_destroy(css_parser, true);
            lxb_html_document_destroy(document);
            return;
        }

        println!("Searching for : '{}'", selector_str);

        let mut match_count: i32 = 0;

        let body_node = (*document).body as *mut lxb_dom_node_t;

        let status = lxb_selectors_find(
            selectors,
            body_node,
            selector_list,
            Some(selector_callback),
            &mut match_count as *mut _ as *mut c_void,
        );

        if status != LXB_STATUS_OK as lxb_status_t {
            eprintln!("Failed during selection find");
        } else if match_count == 0 {
            println!("No matches found.");
        }

        lxb_selectors_destroy(selectors, true);
        lxb_css_parser_destroy(css_parser, true);
        lxb_html_document_destroy(document);
    }
}
