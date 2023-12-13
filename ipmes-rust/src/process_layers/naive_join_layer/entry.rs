use crate::match_event::MatchEvent;
use crate::pattern_match::PatternMatch;
use crate::process_layers::naive_join_layer::entry_wrappers::UniqueEntry;
use itertools::Itertools;
use std::cmp::min;

#[derive(Debug)]
pub struct Entry<'p> {
    pub earliest_time: u64,
    /// sorted by input_event.id
    pub match_events: Vec<MatchEvent<'p>>,
    pub match_entities: Vec<Option<u64>>,
    pub hash: u64,
}

impl<'p> Entry<'p> {
    /// Create an entry with infinite timestamp and empty match edges.
    ///
    /// Infinite timestamp avoid this entry being cleaned in windowing process
    pub fn placeholder(num_nodes: usize) -> Self {
        let match_entities = vec![None; num_nodes];
        Self {
            earliest_time: u64::MAX,
            match_events: Vec::new(),
            match_entities,
            hash: 0,
        }
    }

    /// Create match result from match_events. The input matched edges are assumed to be legal.
    ///
    /// **Important**: This function is inefficient and unsafe, and should only be used for
    /// testing purpose.
    pub fn from_match_edges<L>(match_events: L, num_nodes: usize, num_edges: usize) -> Self
    where
        L: Iterator<Item = MatchEvent<'p>>,
    {
        let mut match_events = match_events.collect_vec();
        match_events.sort_by(|a, b| a.input_event.id.cmp(&b.input_event.id));

        let mut match_entities = vec![None; num_nodes];
        let mut mapping = vec![None; num_edges];
        let mut earliest_time = u64::MAX;
        for edge in &match_events {
            match_entities[edge.matched.subject] = Some(edge.input_event.subject);
            match_entities[edge.matched.object] = Some(edge.input_event.object);

            mapping[edge.matched.id] = Some(edge);

            earliest_time = min(edge.input_event.timestamp, earliest_time);
        }

        let hash = UniqueEntry::calc_hash(&mapping);

        Self {
            earliest_time,
            match_events,
            match_entities,
            hash,
        }
    }
}

impl<'p> From<Entry<'p>> for PatternMatch {
    fn from(value: Entry<'_>) -> Self {
        let mut match_events = value.match_events;
        match_events.sort_by(|a, b| a.matched.id.cmp(&b.matched.id));

        let matched_events = match_events
            .into_iter()
            .map(|edge| edge.input_event)
            .collect_vec();

        Self {
            matched_events,
            /// dummy value
            earliest_time: 0,
            /// dummy value
            latest_time: 0
        }
    }
}
