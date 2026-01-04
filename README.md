# Lexbor Rust Bindings for CSS Selection

Unsafe Rust bindings for the [lexbor](https://github.com/lexbor/lexbor) HTML parsing library.

## Prerequisites

You must have the `lexbor` library installed on your system.

## Usage

### Rust Example

To use this library in Rust, you can parse an HTML document and iterate over nodes matching a CSS selector.

```rust
use lexbor_css::HtmlDocument;

fn main() {
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

    for (i, node) in nodes.iter().enumerate() {
        println!("Match #{}: {}", i + 1, node.text_content().unwrap());
    }
}
```