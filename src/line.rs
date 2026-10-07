//! One ingredient line: "- 1 1/2 cups plain flour, sifted".

use crate::qty::{Amount, Style, fmt_amount, parse_amount};
use crate::units::{Unit, parse_unit, tidy};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Qty {
    pub amount: Amount,
    pub unit: Option<Unit>,
}

impl Qty {
    pub fn style(self) -> Style {
        match self.unit {
            None => Style::Count,
            Some(u) if u.is_metric() => Style::Metric,
            Some(_) => Style::Fraction,
        }
    }

    /// The same amount in the unit that reads best for it.
    pub fn tidied(self) -> Qty {
        let Some(unit) = self.unit else { return self };
        let to = tidy(self.amount.high(), unit);
        Qty { amount: self.amount.times(unit.base() / to.base()), unit: Some(to) }
    }

    pub fn write(self) -> String {
        let n = fmt_amount(self.amount, self.style());
        match self.unit {
            None => n,
            Some(u) => {
                // "1 cup" and "1/2 cup", but "1 1/2 cups" and "2 cups".
                let single = n == "1" || n.ends_with("-1") || (n.contains('/') && !n.contains(' '));
                let plural = !single;
                format!("{n} {}", u.name(plural))
            }
        }
    }
}

/// A line split around its amount. Lines with no amount keep everything in
/// `rest` and are passed through untouched.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Indent and any list marker, kept as written.
    pub lead: String,
    pub qty: Option<Qty>,
    /// A second measure in brackets, as in "1 cup (125 g) flour".
    pub also: Option<Qty>,
    pub rest: String,
}

pub fn parse_line(s: &str) -> Line {
    let body = s.trim_start_matches([' ', '\t', '-', '*', '•']);
    let lead = s[..s.len() - body.len()].to_string();
    let Some((amount, used)) = parse_amount(body) else {
        return Line { lead: String::new(), qty: None, also: None, rest: s.to_string() };
    };
    let after = &body[used..];
    let spaced = after.trim_start();
    let (unit, rest) = match parse_unit(spaced) {
        Some((u, n)) => (Some(u), &spaced[n..]),
        None => (None, after),
    };
    let (also, rest) = match bracketed(rest) {
        Some((q, n)) => (Some(q), &rest[n..]),
        None => (None, rest),
    };
    Line { lead, qty: Some(Qty { amount, unit }), also, rest: rest.to_string() }
}

/// " (125 g)" at the start of `s`, with the bytes it took.
fn bracketed(s: &str) -> Option<(Qty, usize)> {
    let inner = s.trim_start().strip_prefix('(')?;
    let (amount, used) = parse_amount(inner)?;
    let after = inner[used..].trim_start();
    let (unit, n) = parse_unit(after)?;
    let close = after[n..].strip_prefix(')')?;
    Some((Qty { amount, unit: Some(unit) }, s.len() - close.len()))
}

/// Things you can't sensibly use half of.
fn whole_only(rest: &str) -> bool {
    let rest = rest.to_ascii_lowercase();
    ["egg", "yolk", "white"].iter().any(|w| rest.split_whitespace().take(3).any(|t| t.starts_with(w)))
}

/// Scales measures in brackets further along, as in "butter (45 g), soft".
fn scale_brackets(s: &str, k: f64) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('(') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        match bracketed(rest) {
            Some((q, n)) => {
                let q = Qty { amount: q.amount.times(k), ..q }.tidied();
                out.push_str(&format!("({})", q.write()));
                rest = &rest[n..];
            }
            None => {
                out.push('(');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

impl Line {
    /// The same line with every amount multiplied by `k`.
    pub fn scaled(&self, k: f64) -> Line {
        let times = |q: Qty| Qty { amount: q.amount.times(k), ..q }.tidied();
        let mut qty = self.qty.map(times);
        if let Some(q) = qty.as_mut().filter(|q| q.unit.is_none() && whole_only(&self.rest)) {
            q.amount = q.amount.whole();
        }
        Line {
            qty,
            also: self.also.map(times),
            rest: scale_brackets(&self.rest, k),
            ..self.clone()
        }
    }

    pub fn write(&self) -> String {
        match self.qty {
            None => self.rest.clone(),
            Some(q) => {
                let also = self.also.map(|a| format!(" ({})", a.write())).unwrap_or_default();
                format!("{}{}{also}{}", self.lead, q.write(), self.rest)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(s: &str) {
        assert_eq!(parse_line(s).write(), s);
    }

    #[test]
    fn splits_the_parts() {
        let l = parse_line("- 1 1/2 cups plain flour, sifted");
        assert_eq!(l.lead, "- ");
        assert_eq!(l.qty, Some(Qty { amount: Amount::One(1.5), unit: Some(Unit::Cup) }));
        assert_eq!(l.rest, " plain flour, sifted");
    }

    #[test]
    fn counts_have_no_unit() {
        let l = parse_line("3 eggs");
        assert_eq!(l.qty.unwrap().unit, None);
        assert_eq!(l.rest, " eggs");
    }

    #[test]
    fn bracketed_second_measure() {
        let l = parse_line("1 cup (125 g) flour");
        assert_eq!(l.also, Some(Qty { amount: Amount::One(125.0), unit: Some(Unit::G) }));
        assert_eq!(l.rest, " flour");
    }

    #[test]
    fn lines_without_amounts_are_untouched() {
        let l = parse_line("Salt and pepper to taste");
        assert_eq!(l.qty, None);
        round_trip("Salt and pepper to taste");
        round_trip("## Icing");
    }

    #[test]
    fn scaling_moves_to_a_better_unit() {
        let l = parse_line("2 tsp vanilla").scaled(3.0);
        assert_eq!(l.write(), "2 tbsp vanilla");
        let l = parse_line("600 g flour").scaled(2.5);
        assert_eq!(l.write(), "1.5 kg flour");
        let l = parse_line("3 tbsp butter (45 g), soft (not melted)").scaled(2.0);
        assert_eq!(l.write(), "6 tbsp butter (90 g), soft (not melted)");
        assert_eq!(parse_line("2 eggs").scaled(1.75).write(), "4 eggs");
        assert_eq!(parse_line("2 large egg yolks").scaled(0.75).write(), "2 large egg yolks");
        assert_eq!(parse_line("2 lemons").scaled(0.75).write(), "1 1/2 lemons");
        let l = parse_line("1/2 cup oil").scaled(0.25);
        assert_eq!(l.write(), "2 tbsp oil");
    }

    #[test]
    fn writes_back_in_house_style() {
        round_trip("- 2 cups milk");
        round_trip("1/2 cup sugar");
        round_trip("1 tbsp oil");
        round_trip("200 g butter (softened)");
        assert_eq!(parse_line("1½ Cups flour").write(), "1 1/2 cups flour");
        assert_eq!(parse_line("200g butter").write(), "200 g butter");
    }
}
