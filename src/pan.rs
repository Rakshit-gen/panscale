//! Pan sizes: "9in round", "23cm round", "8in square", "9x13", "20x30cm".

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Round(f64),
    Square(f64),
    Rect(f64, f64),
}

impl Shape {
    /// Base area in square centimetres. Depth is ignored: a recipe scaled to a
    /// pan with the same depth bakes the same.
    pub fn area(self) -> f64 {
        match self {
            Shape::Round(d) => std::f64::consts::PI * d * d / 4.0,
            Shape::Square(a) => a * a,
            Shape::Rect(a, b) => a * b,
        }
    }
}

/// How much to scale a recipe written for `from` so it fills `to`.
pub fn factor(from: Shape, to: Shape) -> f64 {
    to.area() / from.area()
}

/// Reads a pan description into its shape, in centimetres. Inches are the
/// default unless "cm" appears.
pub fn parse_pan(s: &str) -> Result<Shape, String> {
    let lower = s.to_ascii_lowercase();
    let cm = lower.contains("cm");
    let scale = if cm { 1.0 } else { 2.54 };
    let mut words = Vec::new();
    let mut nums = Vec::new();
    // Split "9x13in" into 9, 13 and in.
    let spaced: String = lower
        .chars()
        .map(|c| if c == 'x' || c == '×' || c == '"' || c == '-' { ' ' } else { c })
        .collect();
    for tok in spaced.split_whitespace() {
        let digits: String = tok.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if !digits.is_empty() {
            nums.push(digits.parse::<f64>().map_err(|_| format!("can't read {tok:?} in {s:?}"))? * scale);
        }
        let word = &tok[digits.len()..];
        if !word.is_empty() {
            words.push(word.to_string());
        }
    }
    let round = words.iter().any(|w| w == "round");
    match (round, nums.as_slice()) {
        (true, [d]) => Ok(Shape::Round(*d)),
        (false, [a]) => Ok(Shape::Square(*a)),
        (false, [a, b]) if a == b => Ok(Shape::Square(*a)),
        (false, [a, b]) => Ok(Shape::Rect(*a, *b)),
        _ => Err(format!("can't read the pan {s:?}; try \"9in round\", \"8in square\" or \"9x13\"")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pan(s: &str) -> Shape {
        parse_pan(s).unwrap()
    }

    #[test]
    fn reads_the_usual_ways_of_writing_a_pan() {
        assert_eq!(pan("9in round"), Shape::Round(9.0 * 2.54));
        assert_eq!(pan("round 9 inch"), Shape::Round(9.0 * 2.54));
        assert_eq!(pan("23cm round"), Shape::Round(23.0));
        assert_eq!(pan("8\" square"), Shape::Square(8.0 * 2.54));
        assert_eq!(pan("20x20cm"), Shape::Square(20.0));
        assert_eq!(pan("9x13"), Shape::Rect(9.0 * 2.54, 13.0 * 2.54));
        assert_eq!(pan("loaf 9 × 5 in"), Shape::Rect(9.0 * 2.54, 5.0 * 2.54));
        assert!(parse_pan("big one").is_err());
        assert!(parse_pan("9x13 round").is_err());
    }

    #[test]
    fn factors() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        // Area grows with the square of the diameter.
        assert!(close(factor(pan("8in round"), pan("9in round")), 81.0 / 64.0));
        assert!(close(factor(pan("8in square"), pan("9x13")), 117.0 / 64.0));
        assert!(close(factor(pan("9x13"), pan("9x13")), 1.0));
    }
}
