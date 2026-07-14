//! Thin wrappers around lexbor's CSS stylesheet parsing engine.
//!
//! Provides access to parsed CSS rules, declarations, and selectors.

use std::marker::PhantomData;
use std::ptr;

use crate::document::HtmlDocument;
use crate::error::Error;
use crate::ffi::{
    lxb_css_parser_create, lxb_css_parser_destroy, lxb_css_parser_init, lxb_css_parser_t,
    lxb_css_stylesheet_create, lxb_css_stylesheet_destroy, lxb_css_stylesheet_t,
    lxb_css_stylesheet_parse, lexbor_status_t_LXB_STATUS_OK,
};

/// A parsed CSS stylesheet.
///
/// The lifetime `'a` is tied to the CSS parser that parsed it.
pub struct StyleSheet<'a> {
    stylesheet: *mut lxb_css_stylesheet_t,
    parser: *mut lxb_css_parser_t,
    _marker: PhantomData<&'a HtmlDocument>,
}

impl<'a> StyleSheet<'a> {
    /// Parse a CSS string into a stylesheet.
    pub fn parse(_doc: &'a HtmlDocument, css: &str) -> Result<Self, Error> {
        unsafe {
            let parser = lxb_css_parser_create();
            if parser.is_null() {
                return Err(Error::CssParserCreate);
            }
            let status = lxb_css_parser_init(parser, ptr::null_mut());
            if status != lexbor_status_t_LXB_STATUS_OK {
                lxb_css_parser_destroy(parser, true);
                return Err(Error::CssParserCreate);
            }

            let stylesheet = lxb_css_stylesheet_create(ptr::null_mut());
            if stylesheet.is_null() {
                lxb_css_parser_destroy(parser, true);
                return Err(Error::CssParserCreate);
            }

            let css_bytes = css.as_bytes();
            let status = lxb_css_stylesheet_parse(
                stylesheet,
                parser,
                css_bytes.as_ptr() as *const crate::ffi::lxb_char_t,
                css_bytes.len(),
            );
            if status != lexbor_status_t_LXB_STATUS_OK {
                lxb_css_stylesheet_destroy(stylesheet, true);
                lxb_css_parser_destroy(parser, true);
                return Err(Error::SelectorParse(css.to_string()));
            }

            Ok(StyleSheet {
                stylesheet,
                parser,
                _marker: PhantomData,
            })
        }
    }

    /// Returns a pointer to the underlying `lxb_css_stylesheet_t`.
    /// For advanced users who need direct access to lexbor's CSS API.
    pub unsafe fn as_ptr(&self) -> *mut lxb_css_stylesheet_t {
        self.stylesheet
    }
}

impl<'a> Drop for StyleSheet<'a> {
    fn drop(&mut self) {
        unsafe {
            lxb_css_stylesheet_destroy(self.stylesheet, true);
            lxb_css_parser_destroy(self.parser, true);
        }
    }
}
