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
