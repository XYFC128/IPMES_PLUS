mod sub_pattern_buffer;

use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use crate::pattern_match::PatternMatch;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::{EarliestFirst, SubPatternMatch};
use log::debug;
use std::collections::{BinaryHeap, HashMap};
pub use sub_pattern_buffer::SubPatternBuffer;

use super::compo_layer;

/// The layer that joins sub-pattern matches into pattern matches.
#[derive(Debug)]
pub struct JoinLayer<'p, P> {
    prev_layer: P,

    /// The behavioral pattern.
    pattern: &'p Pattern,

    /// Binary-tree-structured buffers that store sub-pattern matches.
    ///
    /// A sub-pattern match in a parent node is joined from sub-patterns in its two children buffers.
    sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,

    /// See `clear_expired()`.
    window_size: u64,

    /// Complete pattern matches.
    full_match: Vec<PatternMatch>,
}

impl<'p, P> JoinLayer<'p, P> {
    /// Create and initialize a pair of sub-pattern buffers.
    fn create_buffer_pair(
        buffer_id: usize,
        pattern: &'p Pattern,
        init_buffers: &mut HashMap<usize, SubPatternBuffer>,
    ) {
        let relations = SubPatternBuffer::generate_relations(
            pattern,
            &init_buffers[&buffer_id],
            &init_buffers[&(buffer_id + 1)],
        );

        // update
        init_buffers.get_mut(&buffer_id).unwrap().relation = relations.clone();
        init_buffers.get_mut(&(buffer_id + 1)).unwrap().relation = relations;

        init_buffers.insert(
            get_parent_id(buffer_id),
            SubPatternBuffer::merge_buffers(
                &init_buffers[&buffer_id],
                &init_buffers[&(buffer_id + 1)],
            ),
        );
    }
    pub fn new(
        prev_layer: P,
        pattern: &'p Pattern,
        sub_patterns: &[SubPattern<'p>],
        window_size: u64,
    ) -> Self {
        let buffer_len = 2 * sub_patterns.len() - 1;
        let mut sub_pattern_buffers = Vec::with_capacity(buffer_len);
        let mut init_buffers = HashMap::new();

        for (i, sub_pattern) in sub_patterns.iter().enumerate() {
            let buffer_id = get_buffer_id(i, buffer_len);
            init_buffers.insert(
                buffer_id,
                SubPatternBuffer::new(
                    buffer_id,
                    get_sibling_id(buffer_id),
                    sub_pattern,
                    pattern.entities.len(),
                    pattern.events.len(),
                ),
            );
        }

        for buffer_id in (1..buffer_len - 1).step_by(2).rev() {
            Self::create_buffer_pair(buffer_id, pattern, &mut init_buffers);
        }

        for buffer_id in 0..buffer_len {
            sub_pattern_buffers.push(init_buffers[&buffer_id].clone());
        }

        Self {
            prev_layer,
            pattern,
            sub_pattern_buffers,
            window_size,
            full_match: Vec::new(),
        }
    }

    /// Convert "SubPatternMatch" to "PatternMatch".
    fn pattern_match_conversion(buffer: &mut BinaryHeap<EarliestFirst<'p>>) -> Vec<PatternMatch> {
        let mut pattern_matches = Vec::with_capacity(buffer.len());

        for sub_pattern_match in buffer.drain() {
            debug!("sub_pattern_match id: {}", sub_pattern_match.0.id);
            let mut matched_events = Vec::with_capacity(sub_pattern_match.0.event_ids.len());
            let mut earliest_time = u64::MAX;
            let mut latest_time = u64::MIN;

            for (idx, entry) in sub_pattern_match.0.match_event_map.iter().enumerate() {
                if let Some(event) = entry {
                    for id in event.event_ids.iter() {
                        matched_events.push((idx, *id));
                    }
                    earliest_time = u64::min(earliest_time, event.start_time);
                    latest_time = u64::max(latest_time, event.end_time);
                }
            }
            matched_events.sort_unstable();

            pattern_matches.push(PatternMatch {
                matched_events: matched_events.into_boxed_slice(),
                earliest_time,
                latest_time,
            });
        }
        pattern_matches
    }

    /// Add the sub-pattern matches in the root buffer to the final buffer.
    ///
    /// The uniqueness of matches is handled in the next layer (The Uniqueness Layer).
    fn add_to_answer(&mut self) {
        let root_id = get_root_buffer_id();
        self.full_match.extend(Self::pattern_match_conversion(
            &mut self.sub_pattern_buffers[root_id].buffer,
        ));
        self.full_match.extend(Self::pattern_match_conversion(
            &mut self.sub_pattern_buffers[root_id].new_match_buffer,
        ));
    }

    /// Clear sub-pattern matches whose earliest event goes beyond the window.
    fn clear_expired(&mut self, latest_time: u64, buffer_id: usize) {
        while let Some(sub_pattern_match) = self.sub_pattern_buffers[buffer_id].buffer.peek() {
            if latest_time.saturating_sub(self.window_size) > sub_pattern_match.0.earliest_time {
                self.sub_pattern_buffers[buffer_id].buffer.pop();
            } else {
                break;
            }
        }
    }

    /// Join the new matches of the current buffer (`my_id`) with existing matches in its sibling buffer (`sibling_id`).
    fn join_with_sibling(
        &mut self,
        my_id: usize,
        sibling_id: usize,
    ) -> BinaryHeap<EarliestFirst<'p>> {
        debug!(
            "join with sibling: (my_id, sibling_id) = ({}, {})",
            my_id, sibling_id
        );
        let mut matches_to_parent = BinaryHeap::new();
        let buffer1 = &self.sub_pattern_buffers[my_id].new_match_buffer;
        let buffer2 = &self.sub_pattern_buffers[sibling_id].buffer;

        debug!("my new_match_buffer size: {}", buffer1.len());
        debug!("sibling buffer size: {}", buffer2.len());

        for sub_pattern_match1 in buffer1 {
            for sub_pattern_match2 in buffer2 {
                debug!("***********************************");
                debug!(
                    "now try merging\n{}: {:?} and\n{}: {:?}",
                    sub_pattern_match1.0.id,
                    sub_pattern_match1.0.match_event_map,
                    sub_pattern_match2.0.id,
                    sub_pattern_match2.0.match_event_map
                );

                if let Some(merged) = SubPatternMatch::merge_matches(
                    &self.sub_pattern_buffers[my_id],
                    &sub_pattern_match1.0,
                    &sub_pattern_match2.0,
                ) {
                    matches_to_parent.push(EarliestFirst(merged));
                } else {
                    debug!(
                        "merge {} and {} failed",
                        sub_pattern_match1.0.id, sub_pattern_match2.0.id
                    );
                }
                debug!("***********************************");
            }
        }

        matches_to_parent
    }

    /// Continuously join matches in buffers, in a button-up fashion.
    fn join(&mut self, current_time: u64, mut buffer_id: usize) {
        loop {
            debug!("buffer id: {}", buffer_id);

            // root reached
            if buffer_id == get_root_buffer_id() {
                self.add_to_answer();
                break;
            }

            // Clear only the sibling buffer, since we can clear the current buffer when needed (deferred).
            self.clear_expired(current_time, get_sibling_id(buffer_id));

            let joined = self.join_with_sibling(buffer_id, get_sibling_id(buffer_id));
            let parent_id = get_parent_id(buffer_id);

            self.sub_pattern_buffers[parent_id]
                .new_match_buffer
                .extend(joined);

            // move new matches to buffer
            let new_matches =
                std::mem::take(&mut self.sub_pattern_buffers[buffer_id].new_match_buffer);

            self.sub_pattern_buffers[buffer_id]
                .buffer
                .extend(new_matches);

            if self.sub_pattern_buffers[parent_id]
                .new_match_buffer
                .is_empty()
            {
                debug!("parent buffer no new match! join terminates");
                break;
            } else {
                debug!(
                    "{} new matches pushed to parent",
                    self.sub_pattern_buffers[parent_id].new_match_buffer.len()
                );
            }

            buffer_id = parent_id;
        }
    }
}

/// Node id format conversion.
fn convert_entity_id_map(entity_id_map: &mut Vec<(u64, u64)>, node_ids: &[Option<u64>]) {
    for (i, node_id) in node_ids.iter().enumerate() {
        if let Some(matched_id) = node_id {
            entity_id_map.push((*matched_id, i as u64));
        }
    }

    entity_id_map.sort();
}

/// Convert a vector of events into a vector of event ids indexed by match event id.
fn create_event_id_map(event_id_map: &mut [Option<u64>], events: &Vec<MatchEvent>) {
    for edge in events {
        event_id_map[edge.matched.id] = Some(edge.input_event.event_id);
    }
}

/// Get the corresponding buffer id of a sub-pattern match.
fn get_buffer_id(sub_pattern_id: usize, buffer_len: usize) -> usize {
    sub_pattern_id + buffer_len / 2
}

// /// Get the left buffer id among siblings (might be the buffer indicated by `buffer_id` itself).
// fn get_left_buffer_id(buffer_id: usize) -> usize {
//     buffer_id + buffer_id % 2 - 1
// }

/// Get sibling's buffer id.
///
/// Siblings' buffer ids only differ by their LSB.
fn get_sibling_id(buffer_id: usize) -> usize {
    // root has no sibling
    if buffer_id == get_root_buffer_id() {
        return buffer_id;
    }
    if buffer_id % 2 == 0 {
        buffer_id - 1
    } else {
        buffer_id + 1
    }
}

/// Get parent buffer's id.
fn get_parent_id(buffer_id: usize) -> usize {
    // root has no parent
    if buffer_id == get_root_buffer_id() {
        return buffer_id;
    }
    // get_left_buffer_id(buffer_id) + 2
    (buffer_id - 1) / 2
}

/// Get the root buffer's id.
fn get_root_buffer_id() -> usize {
    0
}

impl<'p, P> Iterator for JoinLayer<'p, P>
where
    P: Iterator<Item = (u32, compo_layer::MatchInstance<'p>)>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        while self.full_match.is_empty() {
            let (sub_pattern_id, match_instance) = self.prev_layer.next()?;

            let num_pat_event = self.pattern.events.len();
            if let Some(sub_match) =
                SubPatternMatch::build(sub_pattern_id, match_instance, num_pat_event)
            {
                let buffer_id = get_buffer_id(sub_match.id, self.sub_pattern_buffers.len());
                let current_time = sub_match.latest_time;
                // put the sub-pattern match to its corresponding buffer
                self.sub_pattern_buffers[buffer_id]
                    .new_match_buffer
                    .push(EarliestFirst(sub_match));

                self.join(current_time, buffer_id);
            }
        }

        self.full_match.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sub_pattern::decompose;
    #[test]
    fn test_generate_sub_pattern_buffers() {
        let pattern = Pattern::parse("../data/universal_patterns/SP8_regex.json")
            .expect("Failed to parse pattern");

        let windows_size = 1800 * 1000;
        let sub_patterns = decompose(&pattern);
        println!("{:#?}", sub_patterns);
        println!("\n\n");
        let join_layer = JoinLayer::new((), &pattern, &sub_patterns, windows_size);
        println!("{:#?}", join_layer.sub_pattern_buffers);
    }
}
