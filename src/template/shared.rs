use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub const DEFAULT_COVER: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='300'%3E%3Crect fill='%23e0e0e0' width='200' height='300'/%3E%3Ctext fill='%23999' x='100' y='150' text-anchor='middle'%3E无封面%3C/text%3E%3C/svg%3E";

pub fn url_encode(s: &str) -> String {
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}