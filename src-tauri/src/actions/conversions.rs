pub struct Conversion {
    pub display: String,
    pub value: String,
}

pub fn convert(input: &str) -> Option<Conversion> {
    let tokens: Vec<_> = input.split_whitespace().collect();
    if tokens.len() != 4 || !matches!(tokens[2].to_ascii_lowercase().as_str(), "to" | "in") {
        return None;
    }
    let value: f64 = tokens[0].replace(',', ".").parse().ok()?;
    let from = normalize_unit(tokens[1]);
    let to = normalize_unit(tokens[3]);
    let converted = match (from.as_str(), to.as_str()) {
        ("km", "mi") => value * 0.621_371_192_2,
        ("mi", "km") => value / 0.621_371_192_2,
        ("m", "ft") => value * 3.280_839_895,
        ("ft", "m") => value / 3.280_839_895,
        ("c", "f") => value * 9.0 / 5.0 + 32.0,
        ("f", "c") => (value - 32.0) * 5.0 / 9.0,
        ("gb", "mb") => value * 1024.0,
        ("mb", "gb") => value / 1024.0,
        ("kg", "lb") => value * 2.204_622_621_8,
        ("lb", "kg") => value / 2.204_622_621_8,
        _ => return None,
    };
    let formatted = if converted.fract().abs() < 1e-10 {
        format!("{converted:.0}")
    } else {
        format!("{converted:.6}").trim_end_matches('0').to_owned()
    };
    let result = format!("{formatted} {to}");
    Some(Conversion {
        display: format!("{value} {from} = {result}"),
        value: result,
    })
}

fn normalize_unit(unit: &str) -> String {
    match unit.trim().to_ascii_lowercase().as_str() {
        "miles" | "mile" => "mi",
        "feet" | "foot" => "ft",
        "celsius" => "c",
        "fahrenheit" => "f",
        "kilometers" | "kilometres" => "km",
        "pounds" | "pound" => "lb",
        value => value,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_temperature() {
        assert_eq!(convert("32 f to c").unwrap().value, "0 c");
    }
    #[test]
    fn converts_distance() {
        assert!(
            convert("10 km to miles")
                .unwrap()
                .value
                .starts_with("6.213712")
        );
    }
    #[test]
    fn rejects_currency_without_provider() {
        assert!(convert("10 usd to brl").is_none());
    }
}
