//! Amounts as recipes write them: 2, 1.5, 1/2, 1 1/2.

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
