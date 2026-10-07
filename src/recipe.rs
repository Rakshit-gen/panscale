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
        let Some(word) = YIELD_WORDS.iter().find(|w| lower.starts_with(*w)) else {
            continue;
        };
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

const METHOD_WORDS: [&str; 5] = [
    "method",
    "instructions",
    "directions",
    "steps",
    "preparation",
];

/// The text of a heading line ("## Method", "Method:", "**Directions**"),
/// lowercased, or None for anything else.
fn heading(line: &str) -> Option<String> {
    let t = line.trim().trim_matches(['#', '*', '_', ':', ' ']);
    let marked = line.trim_start().starts_with('#') || line.trim_end().ends_with(':');
    let words = t.split_whitespace().count();
    (marked || words == 1 && !t.is_empty() && t.chars().all(char::is_alphabetic))
        .then(|| t.to_ascii_lowercase())
}

/// Oven temperatures in Fahrenheit rewritten in Celsius, rounded to 5 the
/// way oven dials are: "350°F" and "350 F" become "180°C".
pub fn celsius(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find(|c: char| c.is_ascii_digit()) {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let digits = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let after = &rest[digits..];
        let unit = [
            "°F",
            "° F",
            " °F",
            " F",
            "F",
            " degrees F",
            " degrees Fahrenheit",
        ]
        .iter()
        .find(|u| after.starts_with(*u) && !after[u.len()..].starts_with(char::is_alphanumeric));
        match (unit, rest[..digits].parse::<f64>()) {
            (Some(u), Ok(f)) => {
                let c = ((f - 32.0) * 5.0 / 9.0 / 5.0).round() * 5.0;
                out.push_str(&format!("{c}°C"));
                rest = &after[u.len()..];
            }
            _ => {
                out.push_str(&rest[..digits]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Scales every amount in `text` by `k`, in metric if asked, and leaves everything else as it was.
pub fn scale(text: &str, k: f64, metric: bool) -> String {
    let yields = yield_at(text);
    let mut out = String::with_capacity(text.len());
    let mut method = false;
    for (i, line) in text.lines().enumerate() {
        if let Some(h) = heading(line) {
            method = METHOD_WORDS.contains(&h.as_str());
        }
        match yields {
            // Times and temperatures in the method don't change with size.
            _ if method && metric => out.push_str(&celsius(line)),
            _ if method => out.push_str(line),
            Some((at, a, b, amount)) if at == i => {
                out.push_str(&line[..a]);
                // Nobody serves half a person or bakes half a cookie.
                out.push_str(&fmt_amount(amount.times(k).whole(), Style::Count));
                out.push_str(&line[b..]);
            }
            _ => {
                let mut l = parse_line(line).scaled(k);
                if metric {
                    l = l.metric();
                }
                let l = l.write();
                out.push_str(&if metric { celsius(&l) } else { l });
            }
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
        let out = scale("## Serves 4\n\n- 1 cup rice\n", 1.5, false);
        assert_eq!(out, "## Serves 6\n\n- 1 1/2 cups rice\n");
        assert_eq!(
            scale("Makes 12-14 cookies\n", 2.0, false),
            "Makes 24-28 cookies\n"
        );
        assert_eq!(
            scale("Makes 16 squares\n", 1.83, false),
            "Makes 29 squares\n"
        );
    }

    #[test]
    fn fahrenheit_to_celsius() {
        assert_eq!(
            celsius("Bake at 350°F for 25 minutes."),
            "Bake at 175°C for 25 minutes."
        );
        assert_eq!(
            celsius("Heat the oven to 400 F (or 375F fan)."),
            "Heat the oven to 205°C (or 190°C fan)."
        );
        assert_eq!(celsius("425 degrees F"), "220°C");
        assert_eq!(
            celsius("Use 2 Fuji apples, 180°C"),
            "Use 2 Fuji apples, 180°C"
        );
    }

    #[test]
    fn leaves_the_method_alone() {
        let text = "Ingredients\n- 2 eggs\n\n## Method\nBeat 2 eggs.\n3 minutes on high.\n";
        assert_eq!(
            scale(text, 2.0, false),
            "Ingredients\n- 4 eggs\n\n## Method\nBeat 2 eggs.\n3 minutes on high.\n"
        );
    }
}
