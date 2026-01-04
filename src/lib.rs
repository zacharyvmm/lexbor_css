#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::raw::c_void;
use std::ptr;
use std::marker::PhantomData;

pub struct HtmlDocument {
    document: *mut lxb_html_document_t,
    css_parser: *mut lxb_css_parser_t,
    selectors: *mut lxb_selectors_t,
}

pub struct Node<'a> {
    node: *mut lxb_dom_node_t,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl HtmlDocument {
    pub fn new(html: &str) -> Option<Self> {
        unsafe {
            let document = lxb_html_document_create();
            if document.is_null() {
                return None;
            }

            let html_bytes = html.as_bytes();
            let status = lxb_html_document_parse(document, html_bytes.as_ptr(), html_bytes.len());
            if status != 0 {
                lxb_html_document_destroy(document);
                return None;
            }

            let css_parser = lxb_css_parser_create();
            if css_parser.is_null() {
                lxb_html_document_destroy(document);
                return None;
            }
            let status = lxb_css_parser_init(css_parser, ptr::null_mut());
            if status != 0 {
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
                return None;
            }

            let selectors = lxb_selectors_create();
            if selectors.is_null() {
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
                return None;
            }
            let status = lxb_selectors_init(selectors);
            if status != 0 {
                lxb_selectors_destroy(selectors, true);
                lxb_css_parser_destroy(css_parser, true);
                lxb_html_document_destroy(document);
                return None;
            }

            Some(HtmlDocument {
                document,
                css_parser,
                selectors,
            })
        }
    }

    pub fn select<'a>(&'a self, selector: &str) -> Vec<Node<'a>> {
        unsafe {
            let selector_bytes = selector.as_bytes();
            let selector_list = lxb_css_selectors_parse(
                self.css_parser,
                selector_bytes.as_ptr(),
                selector_bytes.len(),
            );

            if selector_list.is_null() {
                return Vec::new();
            }

            let mut nodes = Vec::new();
            let body = (*self.document).body as *mut lxb_dom_node_t;

            lxb_selectors_find(
                self.selectors,
                body,
                selector_list,
                Some(collect_nodes_callback),
                &mut nodes as *mut _ as *mut c_void,
            );

            nodes
        }
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

impl<'a> Node<'a> {
    pub fn text_content(&self) -> Option<&str> {
        unsafe {
            let mut len: usize = 0;
            let text_ptr = lxb_dom_node_text_content(self.node, &mut len);

            if text_ptr.is_null() {
                return None;
            }

            let text_slice = std::slice::from_raw_parts(text_ptr, len);

            // 1. This is better for security
            // let text_str = String::from_utf8_lossy(text_slice).into_owned();

            // 2. This is faster
            let text_str: &str = str::from_utf8_unchecked(text_slice);

            Some(text_str)
        }
    }
}

unsafe extern "C" fn collect_nodes_callback(
    node: *mut lxb_dom_node_t,
    _spec: lxb_css_selector_specificity_t,
    ctx: *mut c_void,
) -> lxb_status_t {
    unsafe {
        let nodes = &mut *(ctx as *mut Vec<Node>);
        let node_struct = Node {
            node,
            _marker: PhantomData,
        };
        
        nodes.push(node_struct);
    }
    
    0 // LXB_STATUS_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let html = "<div><p>Hello</p><p>World</p></div>";
        let doc = HtmlDocument::new(html).expect("Failed to create document");
        let nodes = doc.select("p");
        
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].text_content(), Some("Hello"));
        assert_eq!(nodes[1].text_content(), Some("World"));
    }

    #[test]
    fn test_select_String() {
        let html = String::from("<div><p>Hello</p><p>World</p></div>");
        let doc = HtmlDocument::new(html.as_str()).expect("Failed to create document");
        let nodes = doc.select("p");
        
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].text_content(), Some("Hello"));
        assert_eq!(nodes[1].text_content(), Some("World"));
    }

    #[test]
    fn test_iteration() {
        let html = r#"`
            <div class="content">
                <p>First paragraph</p>
                <p>Second paragraph</p>
                <span>Some span</span>
            </div>
        "#;

        let doc = HtmlDocument::new(html).expect("Failed to parse HTML");
        let nodes = doc.select("div.content p");

        println!("Found {} nodes:", nodes.len());

        // for node in nodes.iter() {
        //     println!("Matched : {}", node.text_content().unwrap());
        // }

        let mut iter = nodes.iter();
        assert_eq!(iter.next().unwrap().text_content(), Some("First paragraph"));
        assert_eq!(iter.next().unwrap().text_content(), Some("Second paragraph"));
    }
}