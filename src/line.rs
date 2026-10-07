//! One ingredient line: "- 1 1/2 cups plain flour, sifted".

use crate::qty::{Amount, Style, fmt_amount, parse_amount};
use crate::units::{Unit, parse_unit};

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
    pub rest: String,
}

pub fn parse_line(s: &str) -> Line {
    let body = s.trim_start_matches([' ', '\t', '-', '*', '•']);
    let lead = s[..s.len() - body.len()].to_string();
    let Some((amount, used)) = parse_amount(body) else {
        return Line { lead: String::new(), qty: None, rest: s.to_string() };
    };
    let after = &body[used..];
    let spaced = after.trim_start();
    let (unit, rest) = match parse_unit(spaced) {
        Some((u, n)) => (Some(u), &spaced[n..]),
        None => (None, after),
    };
    Line { lead, qty: Some(Qty { amount, unit }), rest: rest.to_string() }
}

impl Line {
    pub fn write(&self) -> String {
        match self.qty {
            None => self.rest.clone(),
            Some(q) => format!("{}{}{}", self.lead, q.write(), self.rest),
        }
    }
}
