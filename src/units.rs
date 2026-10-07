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
            Tsp => 4.928_921_593_75,
            Tbsp => 14.786_764_781_25,
            FlOz => 29.573_529_562_5,
            Cup => 236.588_236_5,
            Pint => 473.176_473,
            Quart => 946.352_946,
            Ml | G => 1.0,
            L | Kg => 1000.0,
            Oz => 28.349_523_125,
            Lb => 453.592_37,
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

/// The unit a cook would pick for `amount` of `unit`, staying in the same
/// family: 6 tsp reads better as 2 tbsp, 1500 g as 1.5 kg. Fluid ounces,
/// pints and quarts are left as they are.
pub fn tidy(amount: f64, unit: Unit) -> Unit {
    use Unit::*;
    let family: &[(Unit, f64)] = match unit {
        // Each unit with the smallest amount (in its own size) it's used for.
        Tsp | Tbsp | Cup => &[(Tsp, 0.0), (Tbsp, 1.0), (Cup, 0.25)],
        Ml | L => &[(Ml, 0.0), (L, 1.0)],
        G | Kg => &[(G, 0.0), (Kg, 1.0)],
        Oz | Lb => &[(Oz, 0.0), (Lb, 1.0)],
        _ => return unit,
    };
    let base = amount * unit.base();
    let smallest = family[0].0;
    family
        .iter()
        .rev()
        .find(|&&(u, min)| {
            let x = base / u.base();
            // 4 tsp stays 4 tsp: nobody measures 1 1/3 tbsp. Metric always moves.
            x >= min - 1e-9 && (u == smallest || u.is_metric() || clean(x, u))
        })
        .map_or(unit, |&(u, _)| u)
}

/// Whether `x` of `u` is something you can measure with the usual set:
/// cups come in thirds and quarters, tablespoons in halves, pounds in quarters.
fn clean(x: f64, u: Unit) -> bool {
    let parts: &[f64] = match u {
        Unit::Cup => &[3.0, 4.0],
        Unit::Lb => &[4.0],
        _ => &[2.0],
    };
    parts.iter().any(|d| ((x * d).round() - x * d).abs() < 0.02)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings() {
        assert_eq!(parse_unit("cups flour"), Some((Unit::Cup, 4)));
        assert_eq!(parse_unit("Tbsp. butter"), Some((Unit::Tbsp, 5)));
        assert_eq!(parse_unit("fl oz milk"), Some((Unit::FlOz, 5)));
        assert_eq!(parse_unit("g sugar"), Some((Unit::G, 1)));
        assert_eq!(parse_unit("dash of salt"), Some((Unit::Pinch, 4)));
    }

    #[test]
    fn capital_t_is_a_tablespoon() {
        assert_eq!(parse_unit("T oil"), Some((Unit::Tbsp, 1)));
        assert_eq!(parse_unit("t salt"), Some((Unit::Tsp, 1)));
    }

    #[test]
    fn words_that_only_start_like_a_unit() {
        assert_eq!(parse_unit("garlic"), None);
        assert_eq!(parse_unit("large eggs"), None);
        assert_eq!(parse_unit("carrots"), None);
        assert_eq!(parse_unit("Tomatoes"), None);
        assert_eq!(parse_unit("ozzy"), None);
    }

    #[test]
    fn tidies_within_a_family() {
        assert_eq!(tidy(6.0, Unit::Tsp), Unit::Tbsp);
        assert_eq!(tidy(2.0, Unit::Tsp), Unit::Tsp);
        assert_eq!(tidy(8.0, Unit::Tbsp), Unit::Cup);
        assert_eq!(tidy(0.125, Unit::Cup), Unit::Tbsp);
        assert_eq!(tidy(1500.0, Unit::G), Unit::Kg);
        assert_eq!(tidy(0.4, Unit::Kg), Unit::G);
        assert_eq!(tidy(24.0, Unit::Oz), Unit::Lb);
        assert_eq!(tidy(3.0, Unit::FlOz), Unit::FlOz);
        assert_eq!(tidy(3.0, Unit::Tsp), Unit::Tbsp);
        assert_eq!(tidy(16.0, Unit::Oz), Unit::Lb);
        assert_eq!(tidy(4.0, Unit::Tsp), Unit::Tsp);
        assert_eq!(tidy(6.0, Unit::Tbsp), Unit::Tbsp);
        assert_eq!(tidy(18.0, Unit::Oz), Unit::Oz);
    }

    #[test]
    fn sizes() {
        assert!((Unit::Tbsp.base() / Unit::Tsp.base() - 3.0).abs() < 0.001);
        assert!((Unit::Cup.base() / Unit::Tbsp.base() - 16.0).abs() < 0.001);
        assert!((Unit::Lb.base() / Unit::Oz.base() - 16.0).abs() < 0.001);
    }
}
