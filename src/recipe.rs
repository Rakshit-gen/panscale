//! A whole recipe as plain text, one line at a time.

use crate::line::parse_line;

/// Scales every amount in `text` by `k` and leaves everything else as it was.
pub fn scale(text: &str, k: f64) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        out.push_str(&parse_line(line).scaled(k).write());
        out.push('\n');
    }
    out
}
