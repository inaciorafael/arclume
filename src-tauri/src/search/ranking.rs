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
        let Some((position, _)) = candidate_positions.find(|(_, actual)| *actual == expected)
        else {
            return typo_score(query, candidate);
        };
        if let Some(previous) = previous {
            score -= (position - previous - 1) as i64 * 12;
        }
        previous = Some(position);
    }
    Some(score - candidate.len() as i64)
}

fn typo_score(query: &str, candidate: &str) -> Option<i64> {
    let maximum = match query.chars().count() {
        0..=2 => return None,
        3..=6 => 1,
        _ => 2,
    };
    let stem = candidate
        .rsplit_once('.')
        .map_or(candidate, |(stem, _)| stem);
    [candidate, stem]
        .into_iter()
        .filter(|value| value.chars().count().abs_diff(query.chars().count()) <= maximum)
        .filter_map(|value| edit_distance(query, value, maximum))
        .min()
        .map(|distance| 3_500 - distance as i64 * 300 - candidate.len() as i64)
}

fn edit_distance(left: &str, right: &str, maximum: usize) -> Option<usize> {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    if left.len().abs_diff(right.len()) > maximum {
        return None;
    }
    let mut previous_previous = vec![0; right.len() + 1];
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (left_index, left_character) in left.iter().enumerate() {
        let mut current = vec![left_index + 1; right.len() + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            let substitution =
                previous[right_index] + usize::from(left_character != right_character);
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(substitution);
            if left_index > 0
                && right_index > 0
                && left_character == &right[right_index - 1]
                && left[left_index - 1] == *right_character
            {
                current[right_index + 1] =
                    current[right_index + 1].min(previous_previous[right_index - 1] + 1);
            }
        }
        previous_previous = previous;
        previous = current;
    }
    (previous[right.len()] <= maximum).then_some(previous[right.len()])
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

    #[test]
    fn tolerates_bounded_typos_and_transpositions() {
        assert!(text_score("spotfiy", "spotify").is_some());
        assert!(text_score("calculdora", "calculadora").is_some());
        assert!(text_score("notes", "notse.txt").is_some());
        assert!(text_score("xyz", "spotify").is_none());
    }
}
