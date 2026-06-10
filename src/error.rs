/// Errors that can occur during HTML parsing, CSS selection, or DOM mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Failed to create the lexbor HTML document.
    DocumentCreate,
    /// Failed to parse the HTML string.
    ParseHtml,
    /// Failed to parse an HTML fragment.
    ParseFragment(String),
    /// Failed to create or initialize the CSS parser.
    CssParserCreate,
    /// Failed to create or initialize the selector engine.
    SelectorsCreate,
    /// Failed to parse a CSS selector string.
    SelectorParse(String),
    /// Failed to set an attribute.
    SetAttribute(String),
    /// Failed to remove an attribute.
    RemoveAttribute(String),
    /// Failed to parse a fragment for insertion.
    InsertFragment(String),
    /// The operation is not valid on the document root.
    RootNode,
    /// The HTML contained invalid UTF-8.
    Utf8,
    /// Unknown fragment context tag.
    UnknownTag(String),
    /// Unknown fragment context namespace.
    UnknownNamespace(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DocumentCreate => write!(f, "failed to create HTML document"),
            Error::ParseHtml => write!(f, "failed to parse HTML"),
            Error::ParseFragment(t) => write!(f, "failed to parse HTML fragment (context: {t})"),
            Error::CssParserCreate => write!(f, "failed to create CSS parser"),
            Error::SelectorsCreate => write!(f, "failed to create selector engine"),
            Error::SelectorParse(s) => write!(f, "failed to parse selector: {s}"),
            Error::SetAttribute(a) => write!(f, "failed to set attribute: {a}"),
            Error::RemoveAttribute(a) => write!(f, "failed to remove attribute: {a}"),
            Error::InsertFragment(s) => write!(f, "failed to parse insert fragment: {s}"),
            Error::RootNode => write!(f, "operation not valid on document root node"),
            Error::Utf8 => write!(f, "HTML contains invalid UTF-8"),
            Error::UnknownTag(t) => write!(f, "unknown tag: {t}"),
            Error::UnknownNamespace(n) => write!(f, "unknown namespace: {n}"),
        }
    }
}

impl std::error::Error for Error {}
