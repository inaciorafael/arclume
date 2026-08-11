pub fn text_score(query: &str, candidate: &str) -> Option<i64> {
    if query.is_empty() {
        return Some(0);
    }
    if candidate == query {
        return Some(10_000);
    }
    if candidate.starts_with(query) {
        return Some(8_000 - candidate.len() as i64);
    }
    if let Some(position) = candidate.find(query) {
        return Some(6_000 - position as i64 * 10 - candidate.len() as i64);
    }

    let mut score = 4_000i64;
    let mut candidate_positions = candidate.char_indices();
    let mut previous = None;
    for expected in query.chars() {
        let (position, _) = candidate_positions.find(|(_, actual)| *actual == expected)?;
        if let Some(previous) = previous {
            score -= (position - previous - 1) as i64 * 12;
        }
        previous = Some(position);
    }
    Some(score - candidate.len() as i64)
}

pub fn acronym_score(query: &str, title: &str) -> Option<i64> {
    if query.is_empty() {
        return None;
    }
    let acronym: String = title
        .split(|character: char| !character.is_alphanumeric())
        .filter_map(|word| word.chars().next())
        .flat_map(char::to_lowercase)
        .collect();
    (acronym == query).then_some(9_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_beats_prefix_and_fuzzy() {
        assert!(text_score("code", "code") > text_score("code", "coder"));
        assert!(text_score("code", "coder") > text_score("code", "visualstudiocode"));
    }

    #[test]
    fn accepts_ordered_abbreviation() {
        assert!(text_score("vsc", "visualstudiocode").is_some());
        assert!(text_score("vsc", "spotify").is_none());
    }

    #[test]
    fn detects_word_acronym() {
        assert_eq!(acronym_score("vsc", "Visual Studio Code"), Some(9_000));
        assert_eq!(acronym_score("vsc", "vscode-config"), None);
    }
}
