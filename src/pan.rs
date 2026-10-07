//! Pan sizes: "9in round", "23cm round", "8in square", "9x13", "20x30cm".

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Round(f64),
    Square(f64),
    Rect(f64, f64),
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
