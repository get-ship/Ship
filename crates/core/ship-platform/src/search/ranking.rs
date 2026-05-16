//! Scoring used by every adapter to rank hits consistently.
//!
//! Heuristic (intentionally crude — cmdk find is interactive, not full-text
//! search):
//!
//! - exact name match               → 1000
//! - prefix match on name           →  500
//! - substring match in name        →  250
//! - substring match in description →  100
//! - no match                       →    0 (caller drops the entry)
//!
//! A recency boost is added on top: max +50 for entries updated within
//! the last day, decaying linearly to 0 over 30 days. Adapters without
//! recency info skip the boost.

use chrono::{DateTime, Duration, Utc};

const SCORE_EXACT: f64 = 1000.0;
const SCORE_PREFIX: f64 = 500.0;
const SCORE_NAME_SUBSTR: f64 = 250.0;
const SCORE_DESC_SUBSTR: f64 = 100.0;
const RECENCY_MAX: f64 = 50.0;
const RECENCY_WINDOW_DAYS: i64 = 30;

/// Score a candidate against `query`. Returns 0 when no part of the
/// candidate matches — caller is expected to drop those entries.
pub fn score_text(query: &str, name: &str, description: Option<&str>) -> f64 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        // No query — every candidate scores equally; let recency / order
        // win. Use a small constant so non-empty candidates still rank
        // above ones that would otherwise hit 0.
        return 1.0;
    }
    let name_lc = name.to_lowercase();
    if name_lc == q {
        return SCORE_EXACT;
    }
    if name_lc.starts_with(&q) {
        return SCORE_PREFIX;
    }
    if name_lc.contains(&q) {
        return SCORE_NAME_SUBSTR;
    }
    if let Some(desc) = description
        && desc.to_lowercase().contains(&q)
    {
        return SCORE_DESC_SUBSTR;
    }
    0.0
}

/// Recency boost: newer entries score higher, up to `RECENCY_MAX`.
/// Entries older than `RECENCY_WINDOW_DAYS` contribute 0.
pub fn recency_boost(updated_at: Option<DateTime<Utc>>) -> f64 {
    let Some(ts) = updated_at else { return 0.0 };
    let age = Utc::now() - ts;
    if age < Duration::zero() {
        return RECENCY_MAX;
    }
    let window = Duration::days(RECENCY_WINDOW_DAYS);
    if age >= window {
        return 0.0;
    }
    let frac = 1.0 - (age.num_seconds() as f64 / window.num_seconds() as f64);
    RECENCY_MAX * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_beats_prefix_beats_substring() {
        let exact = score_text("foo", "foo", None);
        let prefix = score_text("foo", "foobar", None);
        let substr = score_text("foo", "barfoo", None);
        let desc = score_text("foo", "unrelated", Some("contains foo"));
        let none = score_text("foo", "unrelated", Some("nope"));
        assert!(exact > prefix);
        assert!(prefix > substr);
        assert!(substr > desc);
        assert!(desc > none);
        assert_eq!(none, 0.0);
    }

    #[test]
    fn case_insensitive() {
        let a = score_text("Foo", "foo", None);
        let b = score_text("foo", "FOO", None);
        assert_eq!(a, SCORE_EXACT);
        assert_eq!(b, SCORE_EXACT);
    }

    #[test]
    fn empty_query_returns_constant() {
        assert_eq!(score_text("", "anything", None), 1.0);
        assert_eq!(score_text("   ", "anything", None), 1.0);
    }

    #[test]
    fn recency_boost_in_range() {
        let now = Utc::now();
        let fresh = recency_boost(Some(now));
        let half = recency_boost(Some(now - Duration::days(15)));
        let old = recency_boost(Some(now - Duration::days(60)));
        assert!(fresh > half);
        assert!(half > old);
        assert_eq!(old, 0.0);
        assert!(fresh <= RECENCY_MAX + 0.01);
    }
}
