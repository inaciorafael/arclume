use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ADAPTIVE_BOOST: i64 = 1_200;

pub fn adaptive_boost(use_count: i64, last_used: i64) -> i64 {
    let frequency = ((use_count.max(1) as f64).ln_1p() * 180.0) as i64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs() as i64);
    let age_days = now.saturating_sub(last_used) as f64 / 86_400.0;
    let recency = (600.0 * (-age_days / 30.0).exp()) as i64;
    (frequency + recency).clamp(0, MAX_ADAPTIVE_BOOST)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boost_is_bounded_and_frequency_is_monotonic() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(adaptive_boost(10, now) >= adaptive_boost(1, now));
        assert!(adaptive_boost(1_000_000, now) <= MAX_ADAPTIVE_BOOST);
    }
}
