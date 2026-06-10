use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lexbor_css::HtmlDocument;

const SMALL_HTML: &str = "<div class='main'><p>Hello</p><span>World</span></div>";

const MEDIUM_HTML: &str = r#"<html>
<head><title>Test Page</title></head>
<body>
    <div id="header"><h1>Welcome</h1></div>
    <div class="content">
        <p class="intro">This is an introduction paragraph.</p>
        <ul id="nav">
            <li class="active"><a href="/">Home</a></li>
            <li><a href="/about">About</a></li>
            <li><a href="/contact">Contact</a></li>
            <li><a href="/blog">Blog</a></li>
        </ul>
        <div class="article">
            <h2>Article Title</h2>
            <p>First paragraph of the article with <strong>bold</strong> text.</p>
            <p>Second paragraph with <em>italic</em> text and <a href="/link">a link</a>.</p>
            <blockquote>A quote from someone famous.</blockquote>
        </div>
    </div>
    <div id="footer"><p>&copy; 2024</p></div>
</body>
</html>"#;

fn bench_parse_small(c: &mut Criterion) {
    c.bench_function("parse_small", |b| {
        b.iter(|| HtmlDocument::parse(black_box(SMALL_HTML)).unwrap())
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    c.bench_function("parse_medium", |b| {
        b.iter(|| HtmlDocument::parse(black_box(MEDIUM_HTML)).unwrap())
    });
}

fn bench_select_tag(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    c.bench_function("select_tag_p", |b| {
        b.iter(|| black_box(doc.select(black_box("p"))))
    });
}

fn bench_select_class(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    c.bench_function("select_class", |b| {
        b.iter(|| black_box(doc.select(black_box(".active"))))
    });
}

fn bench_select_first(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    c.bench_function("select_first", |b| {
        b.iter(|| black_box(doc.select_first(black_box("p"))))
    });
}

fn bench_select_compound(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    c.bench_function("select_compound", |b| {
        b.iter(|| black_box(doc.select(black_box("div.content p strong"))))
    });
}

fn bench_precompiled_selector(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let sel = doc.compile_selector("p").unwrap();
    c.bench_function("precompiled_select", |b| {
        b.iter(|| black_box(sel.find(&doc)))
    });
}

fn bench_text_extraction(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let nodes = doc.select("p");
    c.bench_function("text_extraction", |b| {
        b.iter(|| {
            for node in &nodes {
                black_box(node.text());
            }
        })
    });
}

fn bench_inner_html(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let nodes = doc.select("div.content");
    c.bench_function("inner_html", |b| {
        b.iter(|| {
            for node in &nodes {
                black_box(node.inner_html());
            }
        })
    });
}

fn bench_attributes(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let nodes = doc.select("a");
    c.bench_function("attributes", |b| {
        b.iter(|| {
            for node in &nodes {
                black_box(node.attributes());
            }
        })
    });
}

fn bench_dom_traversal_children(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let root = doc.select_first("div.content").unwrap();
    c.bench_function("children_traversal", |b| {
        b.iter(|| {
            for child in root.children() {
                black_box(child.tag_name());
            }
        })
    });
}

fn bench_dom_traversal_descendants(c: &mut Criterion) {
    let doc = HtmlDocument::parse(MEDIUM_HTML).unwrap();
    let root = doc.select_first("body").unwrap();
    c.bench_function("descendants_traversal", |b| {
        b.iter(|| {
            for node in root.descendants().filter(|n| n.is_element()) {
                black_box(node.tag_name());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_select_tag,
    bench_select_class,
    bench_select_first,
    bench_select_compound,
    bench_precompiled_selector,
    bench_text_extraction,
    bench_inner_html,
    bench_attributes,
    bench_dom_traversal_children,
    bench_dom_traversal_descendants,
);
criterion_main!(benches);
