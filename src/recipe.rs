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

const METHOD_WORDS: [&str; 5] = ["method", "instructions", "directions", "steps", "preparation"];

/// The text of a heading line ("## Method", "Method:", "**Directions**"),
/// lowercased, or None for anything else.
fn heading(line: &str) -> Option<String> {
    let t = line.trim().trim_matches(['#', '*', '_', ':', ' ']);
    let marked = line.trim_start().starts_with('#') || line.trim_end().ends_with(':');
    let words = t.split_whitespace().count();
    (marked || words == 1 && !t.is_empty() && t.chars().all(char::is_alphabetic))
        .then(|| t.to_ascii_lowercase())
}

/// Scales every amount in `text` by `k` and leaves everything else as it was.
pub fn scale(text: &str, k: f64) -> String {
    let yields = yield_at(text);
    let mut out = String::with_capacity(text.len());
    let mut method = false;
    for (i, line) in text.lines().enumerate() {
        if let Some(h) = heading(line) {
            method = METHOD_WORDS.contains(&h.as_str());
        }
        match yields {
            // Times and temperatures in the method don't change with size.
            _ if method => out.push_str(line),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_yield() {
        assert_eq!(servings("Pancakes\nServes 4\n- 2 eggs"), Some(4.0));
        assert_eq!(servings("**Makes 12-14 cookies**"), Some(13.0));
        assert_eq!(servings("Yield: 2 loaves"), Some(2.0));
        assert_eq!(servings("Servings: 6"), Some(6.0));
        assert_eq!(servings("- 2 eggs\nServe warm."), None);
    }

    #[test]
    fn scales_the_yield_line_too() {
        let out = scale("## Serves 4\n\n- 1 cup rice\n", 1.5);
        assert_eq!(out, "## Serves 6\n\n- 1 1/2 cups rice\n");
        assert_eq!(scale("Makes 12-14 cookies\n", 2.0), "Makes 24-28 cookies\n");
    }

    #[test]
    fn leaves_the_method_alone() {
        let text = "Ingredients\n- 2 eggs\n\n## Method\nBeat 2 eggs.\n3 minutes on high.\n";
        assert_eq!(scale(text, 2.0), "Ingredients\n- 4 eggs\n\n## Method\nBeat 2 eggs.\n3 minutes on high.\n");
    }
}
