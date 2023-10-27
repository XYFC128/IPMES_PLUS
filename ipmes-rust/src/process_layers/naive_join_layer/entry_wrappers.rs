use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::collections::hash_map::DefaultHasher;
use std::iter::zip;
use std::cmp::Ordering;
use std::ops::Deref;
use crate::match_edge::MatchEdge;
use crate::process_layers::naive_join_layer::entry::Entry;

/// A wrapper to [Entry]. Can be put int a HashSet
/// and guarantee the uniqueness of the entries.
///
/// The equality of 2 entries holds when their mapping
/// from the pattern to input graph and the timestamps
/// of the match edges are the same. That is, the input
/// edge with different timestamp are considered different
/// here.
#[derive(Debug)]
pub struct UniqueEntry<'p>(pub Rc<Entry<'p>>);

impl UniqueEntry<'_> {
    /// Calculate hash of the mapping from pattern edge id to MatchEdge
    ///
    /// Hash the timestamp and id of input edge.
    pub fn calc_hash(mapping: &[Option<&MatchEdge<'_>>]) -> u64 {
        let mut hasher = DefaultHasher::new();
        let state = &mut hasher;
        for edge in mapping {
            if let Some(match_edge) = edge {
                match_edge.input_edge.timestamp.hash(state);
                match_edge.input_edge.id.hash(state);
            } else {
                state.write_u8(0);
            }
        }
        hasher.finish()
    }
}

impl Eq for UniqueEntry<'_> {}

impl PartialEq for UniqueEntry<'_> {
    /// The equality of 2 entries holds when their mapping
    /// from the pattern to input graph are the same.
    ///
    /// Since `match_edges` in [Entry] are sorted by their id of input edge, 2 entries with same
    /// mapping will contain at least the same set of [MatchEdge]s, which results in the same order
    /// in `match_edges`.
    fn eq(&self, other: &Self) -> bool {
        if self.0.match_edges.len() != other.0.match_nodes.len() {
            return false;
        }
        zip(&self.0.match_edges, &other.0.match_edges).all(|(e1, e2)| {
            e1.matched.id == e2.matched.id && e1.input_edge.timestamp == e2.input_edge.timestamp
        })
    }
}

impl Hash for UniqueEntry<'_> {
    /// Because Entry always unchanged since it's creation, to improve the performance,
    /// it caches its hash in the [Entry::hash] field. For this reason, the real hash function
    /// should refer to [UniqueEntry::calc_hash()]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash.hash(state);
    }
}

/// A wrapper for [Entry]. Order the entries by their earliest
/// timestamp and the earliest entry has the highest priority
/// in a priority queue.
#[derive(Debug)]
pub struct EarliestFirst<'p>(pub Rc<Entry<'p>>);

impl Eq for EarliestFirst<'_> {}

impl PartialEq<Self> for EarliestFirst<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'p> AsRef<Entry<'p>> for EarliestFirst<'p> {
    fn as_ref(&self) -> &Entry<'p> {
        self.0.as_ref()
    }
}