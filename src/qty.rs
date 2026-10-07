//! Amounts as recipes write them: 2, 1.5, 1/2, 1 1/2, 2-3.

/// One amount, or a range like "2-3 cloves".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Amount {
    One(f64),
    Range(f64, f64),
}

impl Amount {
    pub fn times(self, k: f64) -> Amount {
        match self {
            Amount::One(x) => Amount::One(x * k),
            Amount::Range(a, b) => Amount::Range(a * k, b * k),
        }
    }

    /// The larger end, which is what unit choices are based on.
    pub fn high(self) -> f64 {
        match self {
            Amount::One(x) => x,
            Amount::Range(_, b) => b,
        }
    }
}

/// Reads an amount or a range at the start of `s`: "2", "2-3", "2 – 3",
/// "2 to 3", "1/2-1". Returns it and the number of bytes used.
pub fn parse_amount(s: &str) -> Option<(Amount, usize)> {
    let (a, used) = parse_number(s)?;
    let rest = &s[used..];
    for sep in ["-", "–", " - ", " – ", " to ", " or "] {
        if let Some(after) = rest.strip_prefix(sep) {
            if let Some((b, n)) = parse_number(after) {
                if b > a {
                    return Some((Amount::Range(a, b), used + sep.len() + n));
                }
            }
        }
    }
    Some((Amount::One(a), used))
}

const VULGAR: [(char, f64); 15] = [
    ('½', 0.5),
    ('⅓', 1.0 / 3.0),
    ('⅔', 2.0 / 3.0),
    ('¼', 0.25),
    ('¾', 0.75),
    ('⅕', 0.2),
    ('⅖', 0.4),
    ('⅗', 0.6),
    ('⅘', 0.8),
    ('⅙', 1.0 / 6.0),
    ('⅚', 5.0 / 6.0),
    ('⅛', 0.125),
    ('⅜', 0.375),
    ('⅝', 0.625),
    ('⅞', 0.875),
];

/// A single fraction character like ½ at the start of `s`.
fn vulgar(s: &str) -> Option<(f64, usize)> {
    let c = s.chars().next()?;
    VULGAR.iter().find(|(v, _)| *v == c).map(|&(_, x)| (x, c.len_utf8()))
}

/// Reads an amount at the start of `s` and returns it with the number of
/// bytes it took. "1 1/2 cups" gives (1.5, 5).
pub fn parse_number(s: &str) -> Option<(f64, usize)> {
    if let Some(v) = vulgar(s) {
        return Some(v);
    }
    let (whole, mut used) = simple(s)?;
    // "1½" or "1 ½".
    if whole.fract() == 0.0 && !s[..used].contains(['/', '.']) {
        let rest = &s[used..];
        let gap = rest.len() - rest.trim_start_matches(' ').len();
        if let Some((frac, n)) = vulgar(&rest[gap..]) {
            return Some((whole + frac, used + gap + n));
        }
    }
    // "1 1/2": a whole number followed by a fraction.
    if whole.fract() == 0.0 && !s[..used].contains('/') {
        let rest = &s[used..];
        let gap = rest.len() - rest.trim_start_matches(' ').len();
        if gap > 0 {
            if let Some((frac, n)) = simple(&rest[gap..]) {
                if rest[gap..gap + n].contains('/') && frac < 1.0 {
                    used += gap + n;
                    return Some((whole + frac, used));
                }
            }
        }
    }
    Some((whole, used))
}

/// One number: 2, 1.5, .5 or 3/4.
fn simple(s: &str) -> Option<(f64, usize)> {
    let digits = |from: usize| s[from..].find(|c: char| !c.is_ascii_digit()).map_or(s.len(), |n| from + n);
    let end = digits(0);
    if s[end..].starts_with('/') && end > 0 {
        let den_end = digits(end + 1);
        let num: f64 = s[..end].parse().ok()?;
        let den: f64 = s[end + 1..den_end].parse().ok()?;
        return (den != 0.0).then_some((num / den, den_end));
    }
    if s[end..].starts_with('.') && s[end + 1..].starts_with(|c: char| c.is_ascii_digit()) {
        let frac_end = digits(end + 1);
        return Some((s[..frac_end].parse().ok()?, frac_end));
    }
    if end == 0 {
        return None;
    }
    Some((s[..end].parse().ok()?, end))
}

/// How an amount should be written back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Cups and spoons: the nearest half, third, quarter or eighth.
    Fraction,
    /// Grams and millilitres: whole numbers, rounded harder as they grow.
    Metric,
    /// Eggs, cloves, lemons: the nearest half.
    Count,
}

/// The fraction closest to `x`, as (whole, numerator, denominator).
fn nearest_fraction(x: f64) -> (u64, u64, u64) {
    let whole = x.floor();
    let frac = x - whole;
    let mut best = (0u64, 1u64, frac.abs());
    for den in [2u64, 3, 4, 8] {
        let num = (frac * den as f64).round() as u64;
        let err = (frac - num as f64 / den as f64).abs();
        // Prefer the simpler fraction unless the finer one is clearly closer.
        if err + 1e-9 < best.2 {
            best = (num, den, err);
        }
    }
    let (num, den, _) = best;
    if num == den { (whole as u64 + 1, 0, 1) } else { (whole as u64, num, den) }
}

pub fn fmt_number(x: f64, style: Style) -> String {
    match style {
        Style::Fraction | Style::Count => {
            let (whole, num, den) = if style == Style::Count {
                let halves = (x * 2.0).round() as u64;
                (halves / 2, halves % 2, 2)
            } else {
                nearest_fraction(x)
            };
            let g = gcd(num, den);
            let (num, den) = (num / g.max(1), den / g.max(1));
            match (whole, num) {
                (0, 0) => "0".into(),
                (w, 0) => w.to_string(),
                (0, n) => format!("{n}/{den}"),
                (w, n) => format!("{w} {n}/{den}"),
            }
        }
        Style::Metric => {
            let step = match x {
                x if x < 10.0 => 0.5,
                x if x < 100.0 => 1.0,
                x if x < 1000.0 => 5.0,
                _ => 10.0,
            };
            let r = (x / step).round() * step;
            if r.fract() == 0.0 { format!("{}", r as u64) } else { format!("{r:.1}") }
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

pub fn fmt_amount(a: Amount, style: Style) -> String {
    match a {
        Amount::One(x) => fmt_number(x, style),
        Amount::Range(lo, hi) => {
            let (lo, hi) = (fmt_number(lo, style), fmt_number(hi, style));
            if lo == hi { lo } else { format!("{lo}-{hi}") }
        }
    }
}
