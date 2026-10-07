//! Amounts as recipes write them: 2, 1.5, 1/2, 1 1/2.

/// Reads an amount at the start of `s` and returns it with the number of
/// bytes it took. "1 1/2 cups" gives (1.5, 5).
pub fn parse_number(s: &str) -> Option<(f64, usize)> {
    let (whole, mut used) = simple(s)?;
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
