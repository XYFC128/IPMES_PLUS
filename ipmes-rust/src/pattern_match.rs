use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt::Formatter;
use std::fmt::{self};
use std::hash::Hash;

/// Complete Pattern Match
#[derive(Clone, Debug)]
pub struct PatternMatch {
    /// Matched event ids of this pattern. [(pattern_event_id, input_event_id)]
    pub matched_events: Box<[(usize, u64)]>,
    /// The timestamp of the last event (in `matched_events`), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest event; for determining expiry of this match.
    pub earliest_time: u64,
}

impl Hash for PatternMatch {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.matched_events.hash(state);
    }
}

impl Eq for PatternMatch {}

impl PartialEq for PatternMatch {
    fn eq(&self, other: &Self) -> bool {
        self.matched_events == other.matched_events
    }
}

impl fmt::Display for PatternMatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;

        let write_range = |f: &mut Formatter<'_>, start: usize, end: usize| {
            if start + 1 == end {
                write!(f, "{}", self.matched_events[start].1)
            } else {
                write!(
                    f,
                    "({})",
                    self.matched_events[start..end]
                        .iter()
                        .map(|p| p.1)
                        .join(", ")
                )
            }
        };

        if let Some((mut prev_pat_id, _)) = self.matched_events.first() {
            let mut prev_start = 0;
            for (i, (pat_id, _)) in self.matched_events.iter().enumerate().skip(1) {
                if *pat_id != prev_pat_id {
                    write_range(f, prev_start, i)?;
                    write!(f, ", ")?;
                    prev_pat_id = *pat_id;
                    prev_start = i;
                }
            }

            if prev_start < self.matched_events.len() {
                write_range(f, prev_start, self.matched_events.len())?;
            }
        }

        write!(f, "]")
    }
}

/// Helper structure that implements `PartialEq`, `Ord`, `PartialOrd` traits for `PatternMatch`.
///
/// *Earliest* refers to `PatternMatch.earliest_time`.
#[derive(Clone)]
pub struct EarliestFirst(pub PatternMatch);
impl Eq for EarliestFirst {}

impl PartialEq<Self> for EarliestFirst {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_pattern_match(match_events: &[(usize, u64)]) -> String {
        let pat_match = PatternMatch {
            matched_events: match_events.into(),
            earliest_time: 0,
            latest_time: 0,
        };
        pat_match.to_string()
    }

    #[test]
    fn test_formatting() {
        assert_eq!(format_pattern_match(&[]), "[]");
        assert_eq!(format_pattern_match(&[(0, 3), (1, 2), (2, 1)]), "[3, 2, 1]");
        assert_eq!(
            format_pattern_match(&[(0, 3), (1, 2), (1, 1)]),
            "[3, (2, 1)]"
        );
        assert_eq!(
            format_pattern_match(&[(0, 3), (0, 2), (1, 1)]),
            "[(3, 2), 1]"
        );
        assert_eq!(
            format_pattern_match(&[(0, 3), (0, 2), (0, 1)]),
            "[(3, 2, 1)]"
        );
    }
}
