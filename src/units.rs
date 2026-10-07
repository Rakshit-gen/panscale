//! Kitchen units and the words recipes use for them.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Tsp,
    Tbsp,
    FlOz,
    Cup,
    Pint,
    Quart,
    Ml,
    L,
    G,
    Kg,
    Oz,
    Lb,
    Pinch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Volume,
    Mass,
    /// A pinch or dash: no real size, never converted.
    Vague,
}

impl Unit {
    pub fn kind(self) -> Kind {
        use Unit::*;
        match self {
            Tsp | Tbsp | FlOz | Cup | Pint | Quart | Ml | L => Kind::Volume,
            G | Kg | Oz | Lb => Kind::Mass,
            Pinch => Kind::Vague,
        }
    }

    /// Millilitres for volumes, grams for masses (US customary measures).
    pub fn base(self) -> f64 {
        use Unit::*;
        match self {
            Tsp => 4.92892,
            Tbsp => 14.7868,
            FlOz => 29.5735,
            Cup => 236.588,
            Pint => 473.176,
            Quart => 946.353,
            Ml | G => 1.0,
            L | Kg => 1000.0,
            Oz => 28.3495,
            Lb => 453.592,
            Pinch => 0.0,
        }
    }

    pub fn is_metric(self) -> bool {
        matches!(self, Unit::Ml | Unit::L | Unit::G | Unit::Kg)
    }

    /// How the unit is written, singular or plural.
    pub fn name(self, plural: bool) -> &'static str {
        use Unit::*;
        match (self, plural) {
            (Tsp, _) => "tsp",
            (Tbsp, _) => "tbsp",
            (FlOz, _) => "fl oz",
            (Cup, false) => "cup",
            (Cup, true) => "cups",
            (Pint, false) => "pint",
            (Pint, true) => "pints",
            (Quart, false) => "quart",
            (Quart, true) => "quarts",
            (Ml, _) => "ml",
            (L, _) => "l",
            (G, _) => "g",
            (Kg, _) => "kg",
            (Oz, _) => "oz",
            (Lb, _) => "lb",
            (Pinch, false) => "pinch",
            (Pinch, true) => "pinches",
        }
    }
}

/// Every spelling we accept, longest first so "fl oz" wins over "fl".
/// Case matters only for T (tablespoon) and t (teaspoon).
const WORDS: &[(&str, Unit)] = &[
    ("tablespoons", Unit::Tbsp),
    ("tablespoon", Unit::Tbsp),
    ("teaspoons", Unit::Tsp),
    ("teaspoon", Unit::Tsp),
    ("fluid ounces", Unit::FlOz),
    ("fluid ounce", Unit::FlOz),
    ("millilitres", Unit::Ml),
    ("milliliters", Unit::Ml),
    ("millilitre", Unit::Ml),
    ("milliliter", Unit::Ml),
    ("kilograms", Unit::Kg),
    ("kilogram", Unit::Kg),
    ("pinches", Unit::Pinch),
    ("quarts", Unit::Quart),
    ("ounces", Unit::Oz),
    ("pounds", Unit::Lb),
    ("litres", Unit::L),
    ("liters", Unit::L),
    ("grams", Unit::G),
    ("pinch", Unit::Pinch),
    ("pints", Unit::Pint),
    ("quart", Unit::Quart),
    ("ounce", Unit::Oz),
    ("pound", Unit::Lb),
    ("litre", Unit::L),
    ("liter", Unit::L),
    ("gram", Unit::G),
    ("tbsp", Unit::Tbsp),
    ("tbs", Unit::Tbsp),
    ("tsp", Unit::Tsp),
    ("cups", Unit::Cup),
    ("cup", Unit::Cup),
    ("pint", Unit::Pint),
    ("dash", Unit::Pinch),
    ("fl oz", Unit::FlOz),
    ("fl. oz", Unit::FlOz),
    ("lbs", Unit::Lb),
    ("lb", Unit::Lb),
    ("oz", Unit::Oz),
    ("ml", Unit::Ml),
    ("kg", Unit::Kg),
    ("qt", Unit::Quart),
    ("pt", Unit::Pint),
    ("g", Unit::G),
    ("l", Unit::L),
    ("c", Unit::Cup),
];

/// A unit word at the start of `s`, and how many bytes it takes including a
/// trailing full stop ("tbsp."). The word has to end there: "garlic" isn't g.
pub fn parse_unit(s: &str) -> Option<(Unit, usize)> {
    // T and t are the one place case matters.
    for (word, unit) in [("T", Unit::Tbsp), ("t", Unit::Tsp)] {
        if s.starts_with(word) && ends_word(&s[1..]) {
            return Some((unit, 1 + usize::from(s[1..].starts_with('.'))));
        }
    }
    for &(word, unit) in WORDS {
        let head = s.get(..word.len());
        if head.is_some_and(|h| h.eq_ignore_ascii_case(word)) && ends_word(&s[word.len()..]) {
            let dot = usize::from(s[word.len()..].starts_with('.'));
            return Some((unit, word.len() + dot));
        }
    }
    None
}

fn ends_word(rest: &str) -> bool {
    rest.chars().next().is_none_or(|c| !c.is_alphanumeric())
}
