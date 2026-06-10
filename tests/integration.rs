use lexbor_css::{HtmlDocument, TextOpts};

// =========================================================================
// Core features (17 original tests — all pass)
// =========================================================================

#[test]
fn test_basic_select() {
    let doc = HtmlDocument::parse("<div><p>Hello</p><p>World</p></div>").unwrap();
    let nodes = doc.select("p");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].text(), "Hello");
    assert_eq!(nodes[1].text(), "World");
}

#[test]
fn test_select_from_string() {
    let html = String::from("<div><p>Hello</p><p>World</p></div>");
    let doc = HtmlDocument::parse(&html).unwrap();
    let nodes = doc.select("p");
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_iteration() {
    let html = r#"
        <div class="content">
            <p>First paragraph</p>
            <p>Second paragraph</p>
            <span>Some span</span>
        </div>
    "#;
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("div.content p");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].text(), "First paragraph");
    assert_eq!(nodes[1].text(), "Second paragraph");
}

#[test]
fn test_inner_html() {
    let html = r#"<div id="parent"><p>Child 1</p><span>Child 2</span></div>"#;
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("#parent");
    assert_eq!(nodes.len(), 1);
    let inner = nodes[0].inner_html();
    assert_eq!(inner, "<p>Child 1</p><span>Child 2</span>");
}

#[test]
fn test_attributes() {
    let html = r#"<div id="my-div" class="container" data-val="123">Content</div>"#;
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("div");
    assert_eq!(nodes.len(), 1);
    let attrs = nodes[0].attributes();
    assert_eq!(attrs.len(), 3);
    assert_eq!(attrs["id"], "my-div");
    assert_eq!(attrs["class"], "container");
    assert_eq!(attrs["data-val"], "123");
}

#[test]
fn test_select_first() {
    let html = "<ul><li>A</li><li>B</li><li>C</li></ul>";
    let doc = HtmlDocument::parse(html).unwrap();
    let node = doc.select_first("li").unwrap();
    assert_eq!(node.text(), "A");
}

#[test]
fn test_select_first_none() {
    let html = "<div></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    assert!(doc.select_first("span").is_none());
}

#[test]
fn test_tag_name() {
    let html = "<div class='x'><p>Hi</p></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("p");
    assert_eq!(nodes[0].tag_name(), "p");
}

#[test]
fn test_id_and_class() {
    let html = r#"<div id="hero" class="main content">text</div>"#;
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("#hero");
    assert_eq!(nodes[0].id(), Some("hero"));
    assert_eq!(nodes[0].class(), Some("main content"));
    assert!(nodes[0].has_class("main"));
    assert!(nodes[0].has_class("content"));
    assert!(!nodes[0].has_class("nope"));
    assert_eq!(nodes[0].classes(), vec!["main", "content"]);
}

#[test]
fn test_attr() {
    let html = r#"<a href="/home" target="_blank">Home</a>"#;
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("a");
    assert_eq!(nodes[0].attr("href"), Some("/home"));
    assert_eq!(nodes[0].attr("target"), Some("_blank"));
    assert!(nodes[0].has_attr("href"));
    assert!(!nodes[0].has_attr("rel"));
}

#[test]
fn test_outer_html() {
    let html = "<div><span>x</span></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("span");
    assert_eq!(nodes[0].outer_html(), "<span>x</span>");
}

#[test]
fn test_dom_traversal() {
    let html = "<div id='root'><p>A</p><p>B</p></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let root = doc.select_first("#root").unwrap();

    let kids: Vec<_> = root.children().map(|n| n.tag_name()).collect();
    assert_eq!(kids, vec!["p", "p"]);

    let first = root.first_child().unwrap();
    assert_eq!(first.tag_name(), "p");
    assert_eq!(first.text(), "A");

    let second = first.next_sibling().unwrap();
    assert_eq!(second.text(), "B");

    assert_eq!(first.parent().unwrap().tag_name(), "div");

    let desc: Vec<_> = root
        .descendants()
        .filter(|n| n.is_element())
        .map(|n| n.tag_name())
        .collect();
    assert_eq!(desc, vec!["p", "p"]);
}

#[test]
fn test_empty_document() {
    let doc = HtmlDocument::parse("").unwrap();
    assert!(doc.select("div").is_empty());
    assert!(doc.select_first("div").is_none());
}

#[test]
fn test_selector_class() {
    let html = "<div><p class='foo'>A</p><p class='bar'>B</p><p class='foo'>C</p></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select(".foo");
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].text(), "A");
    assert_eq!(nodes[1].text(), "C");
}

#[test]
fn test_selector_descendant() {
    let html = "<div><ul><li>1</li></ul><li>2</li></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("div li");
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_selector_child_combinator() {
    let html = "<div><ul><li>1</li></ul><li>2</li></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let nodes = doc.select("div > li");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].text(), "2");
}

#[test]
fn test_precompiled_selector() {
    let html = "<div><p>A</p><p>B</p></div>";
    let doc = HtmlDocument::parse(html).unwrap();
    let sel = doc.compile_selector("p").unwrap();
    let nodes = sel.find(&doc);
    assert_eq!(nodes.len(), 2);
}

// =========================================================================
// New features that work
// =========================================================================

#[test]
fn test_node_type_checks() {
    let doc = HtmlDocument::parse("<div><p>Hello</p></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.is_element());
    assert!(!div.is_text());
    assert!(!div.is_document());

    let p = doc.select_first("p").unwrap();
    let text_node = p.first_child().unwrap();
    assert!(text_node.is_text());
}

#[test]
fn test_tag_id() {
    let doc = HtmlDocument::parse("<div><p>Hello</p><span>World</span></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    let p = doc.select_first("p").unwrap();
    assert_ne!(div.tag_id(), p.tag_id());
}

#[test]
fn test_text_content_direct() {
    let doc = HtmlDocument::parse("<p>Hello <b>World</b></p>").unwrap();
    let p = doc.select_first("p").unwrap();
    assert_eq!(p.text(), "Hello World");
    let text_child = p.first_child().unwrap();
    assert!(text_child.is_text());
    assert_eq!(text_child.text_content(), Some("Hello "));
}

#[test]
fn test_is_empty_text_node() {
    let doc = HtmlDocument::parse("<div>   </div>").unwrap();
    let div = doc.select_first("div").unwrap();
    let text_node = div.first_child().unwrap();
    assert!(text_node.is_text());
    assert!(text_node.is_empty_text_node());
}

#[test]
fn test_text_with_options() {
    let doc = HtmlDocument::parse("<div><p>Hello</p><p>World</p></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    let opts = TextOpts { deep: false, ..TextOpts::default() };
    assert_eq!(div.text_with(opts), "");
}

#[test]
fn test_decompose() {
    let doc = HtmlDocument::parse("<div><p>Hello</p><span>World</span></div>").unwrap();
    let p = doc.select_first("p").unwrap();
    assert!(p.decompose().is_ok());
    assert_eq!(doc.select("p").len(), 0);
    assert_eq!(doc.select("span").len(), 1);
}

#[test]
fn test_decompose_shallow() {
    let doc = HtmlDocument::parse("<div><p><b>Hello</b></p><span>World</span></div>").unwrap();
    let p = doc.select_first("p").unwrap();
    assert!(p.decompose_shallow().is_ok());
    assert_eq!(doc.select("p").len(), 0);
    assert_eq!(doc.select("b").len(), 0);
}

#[test]
fn test_set_attr() {
    let doc = HtmlDocument::parse("<div></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.set_attr("id", Some("my-id")).is_ok());
    assert_eq!(div.attr("id"), Some("my-id"));
}

#[test]
fn test_remove_attr() {
    let doc = HtmlDocument::parse("<div id='x' class='y'>text</div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.remove_attr("id").is_ok());
    assert_eq!(div.attr("id"), None);
}

#[test]
fn test_append_text() {
    let doc = HtmlDocument::parse("<div></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.append_text("Hello").is_ok());
    assert!(div.append_text("World").is_ok());
    assert_eq!(div.text(), "HelloWorld");
}

#[test]
fn test_tags() {
    let doc = HtmlDocument::parse("<div><p>A</p><p>B</p><span>C</span></div>").unwrap();
    assert_eq!(doc.tags("p").len(), 2);
    assert_eq!(doc.tags("span").len(), 1);
}

#[test]
fn test_matches() {
    let doc = HtmlDocument::parse("<div><p>Hi</p></div>").unwrap();
    assert!(doc.matches("p"));
    assert!(!doc.matches("span"));
}

#[test]
fn test_strip_tags() {
    let doc = HtmlDocument::parse("<div><script>evil()</script><p>Hello</p></div>").unwrap();
    doc.strip_tags(&["script"]);
    assert_eq!(doc.select("script").len(), 0);
}

#[test]
fn test_css_matches() {
    let doc = HtmlDocument::parse("<div class='main'><p>Hi</p></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.css_matches("div"));
    assert!(!div.css_matches("p"));
}

#[test]
fn test_any_css_matches() {
    let doc = HtmlDocument::parse("<div class='x'><p>Hi</p></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    assert!(div.any_css_matches(&["span", "div"]));
}

#[test]
fn test_selector_find_first_from() {
    let doc = HtmlDocument::parse("<div><p>A</p><p>B</p></div>").unwrap();
    let sel = doc.compile_selector("p").unwrap();
    let div = doc.select_first("div").unwrap();
    let first = sel.find_first_from(&doc, &div).unwrap();
    assert_eq!(first.text(), "A");
}

#[test]
fn test_traverse() {
    let doc = HtmlDocument::parse("<div><p>A</p><p>B</p></div>").unwrap();
    let div = doc.select_first("div").unwrap();
    let tags: Vec<_> = div.traverse().filter(|n| n.is_element()).map(|n| n.tag_name()).collect();
    assert_eq!(tags, vec!["div", "p", "p"]);
}

#[test]
fn test_iter_children_with_text() {
    let doc = HtmlDocument::parse("<div>Hello <b>World</b>!</div>").unwrap();
    let div = doc.select_first("div").unwrap();
    let kids = div.children().filter(|n| n.is_element()).count();
    assert_eq!(kids, 1);
}
