#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::os::raw::c_void;
use std::ptr;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::str;

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

            let text_str: &str = str::from_utf8_unchecked(text_slice);

            Some(text_str)
        }
    }

    // To get the innerHtml you are forced to serialise the tree
    pub fn inner_html(&self) -> String {
        unsafe {
            let mut result = String::new();
            let mut child = (*self.node).first_child;

            while !child.is_null() {
                lxb_html_serialize_tree_cb(
                    child,
                    Some(serialize_callback),
                    &mut result as *mut String as *mut c_void,
                );
                child = (*child).next;
            }
            result
        }
    }

    pub fn attributes(&self) -> HashMap<&str, &str> {
        let mut attrs = HashMap::new();
        unsafe {
            if (*self.node).type_ != lxb_dom_node_type_t_LXB_DOM_NODE_TYPE_ELEMENT {
                return attrs;
            }

            let element = self.node as *mut lxb_dom_element_t;
            let mut attr = lxb_dom_element_first_attribute_noi(element);

            while !attr.is_null() {
                let mut name_len: usize = 0;
                let name_ptr = lxb_dom_attr_qualified_name(attr, &mut name_len);

                let mut value_len: usize = 0;
                let value_ptr = lxb_dom_attr_value_noi(attr, &mut value_len);

                if !name_ptr.is_null() && !value_ptr.is_null() {
                    let name_slice = std::slice::from_raw_parts(name_ptr, name_len);
                    let value_slice = std::slice::from_raw_parts(value_ptr, value_len);

                    let name: &str = str::from_utf8_unchecked(name_slice);
                    let value: &str = str::from_utf8_unchecked(value_slice);

                    attrs.insert(name, value);
                }

                attr = lxb_dom_element_next_attribute_noi(attr);
            }
        }
        attrs
    }
}

const LXB_STATUS_OK: lxb_status_t = 0;

unsafe extern "C" fn serialize_callback(
    data: *const lxb_char_t,
    len: usize,
    ctx: *mut c_void,
) -> lxb_status_t {
    unsafe {
        let result = &mut *(ctx as *mut String);
        let slice = std::slice::from_raw_parts(data, len);
        if let Ok(s) = str::from_utf8(slice) {
            result.push_str(s);
        }
        LXB_STATUS_OK
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
    
    LXB_STATUS_OK
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

    #[test]
    fn test_inner_html() {
        let html = r#"<div id="parent"><p>Child 1</p><span>Child 2</span></div>"#;
        let doc = HtmlDocument::new(html).expect("Failed to parse HTML");
        let nodes = doc.select("#parent");
        
        assert_eq!(nodes.len(), 1);
        let inner = nodes[0].inner_html();
        assert_eq!(inner, "<p>Child 1</p><span>Child 2</span>");
    }

    #[test]
    fn test_attributes() {
        let html = r#"<div id="my-div" class="container" data-val="123">Content</div>"#;
        let doc = HtmlDocument::new(html).expect("Failed to parse HTML");
        let nodes = doc.select("div");
        
        assert_eq!(nodes.len(), 1);
        let attrs = nodes[0].attributes();
        
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs["id"], "my-div");
        assert_eq!(attrs["class"], "container");
        assert_eq!(attrs["data-val"], "123");
    }
}