//! A whole recipe as plain text, one line at a time.

use crate::line::parse_line;
use crate::qty::{Amount, Style, fmt_amount, parse_amount};

const YIELD_WORDS: [&str; 5] = ["serves", "servings:", "makes", "yield:", "yields"];

/// Where a "Serves 4" or "Makes 12-14" line keeps its number: the line index,
/// the byte range of the number, and the amount.
fn yield_at(text: &str) -> Option<(usize, usize, usize, Amount)> {
    for (i, line) in text.lines().enumerate() {
        let t = line.trim_start_matches([' ', '\t', '#', '*', '_']);
        let lower = t.to_ascii_lowercase();
        let Some(word) = YIELD_WORDS.iter().find(|w| lower.starts_with(*w)) else { continue };
        let after = &t[word.len()..];
        let num = after.trim_start();
        if let Some((amount, n)) = parse_amount(num) {
            let start = line.len() - num.len();
            return Some((i, start, start + n, amount));
        }
    }
    None
}

/// How many the recipe serves or makes, if it says. A range counts as its
/// middle.
pub fn servings(text: &str) -> Option<f64> {
    yield_at(text).map(|(.., a)| match a {
        Amount::One(x) => x,
        Amount::Range(lo, hi) => (lo + hi) / 2.0,
    })
}

/// Scales every amount in `text` by `k` and leaves everything else as it was.
pub fn scale(text: &str, k: f64) -> String {
    let yields = yield_at(text);
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.lines().enumerate() {
        match yields {
            Some((at, a, b, amount)) if at == i => {
                out.push_str(&line[..a]);
                out.push_str(&fmt_amount(amount.times(k), Style::Count));
                out.push_str(&line[b..]);
            }
            _ => out.push_str(&parse_line(line).scaled(k).write()),
        }
        out.push('\n');
    }
    out
}
