mod sub_pattern_buffer;
mod sub_pattern_match;

use crate::pattern::Pattern;
use crate::pattern::SubPattern;
use crate::pattern_match::PatternMatch;
use itertools::Itertools;
use log::debug;
use std::collections::{BinaryHeap, HashMap};
pub use sub_pattern_buffer::SubPatternBuffer;
use sub_pattern_match::EarliestFirst;
pub use sub_pattern_match::SubPatternMatch;

use super::composition_layer;
use super::composition_layer::MatchInstance;

/// The layer that joins sub-pattern matches into pattern matches.
#[derive(Debug)]
pub struct JoinLayer<'p, P> {
    prev_layer: P,

    /// The behavioral pattern.
    pattern: &'p Pattern,

    /// Binary-tree-structured buffers that store sub-pattern matches.
    ///
    /// A sub-pattern match in a parent node is joined from sub-patterns in its two children buffers.
    // sub_pattern_buffers: Vec<SubPatternBuffer<'p>>,
    sub_pattern_buffers: Vec<SubPatternBuffer>,

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
            debug!("pattern.entities.len(): {}", pattern.entities.len());
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
    // fn pattern_match_conversion(buffer: &mut BinaryHeap<EarliestFirst<'p>>) -> Vec<PatternMatch> {
    fn pattern_match_conversion(buffer: &mut BinaryHeap<EarliestFirst>) -> Vec<PatternMatch> {
        let mut pattern_matches = Vec::with_capacity(buffer.len());

        for sub_pattern_match in buffer.drain() {
            debug!("sub_pattern_match id: {}", sub_pattern_match.0.id);
            let mut matched_events = Vec::with_capacity(sub_pattern_match.0.event_ids.len());
            let mut earliest_time = u64::MAX;
            let mut latest_time = u64::MIN;

            for (idx, event) in sub_pattern_match
                .0
                .match_event_map
                .iter()
                .flatten()
                .enumerate()
            {
                // for id in event.event_ids.iter() {
                //     matched_events.push((idx, *id));
                // }
                matched_events.extend(event.raw_events.get_ids().map(|id| (idx, id)));

                let (start_time, end_time) = event.raw_events.get_interval();

                earliest_time = u64::min(earliest_time, start_time);
                latest_time = u64::max(latest_time, end_time);
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
        debug!("windowing...");
        while let Some(sub_pattern_match) = self.sub_pattern_buffers[buffer_id].buffer.peek() {
            debug!("earliest_time: {}", sub_pattern_match.0.earliest_time);
            if latest_time.saturating_sub(self.window_size) > sub_pattern_match.0.earliest_time {
                debug!(
                    "clear expired! (sub_pattern time: {}, latest_time: {}; buffer id: {}))",
                    sub_pattern_match.0.earliest_time, latest_time, buffer_id
                );
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
        // ) -> BinaryHeap<EarliestFirst<'p>> {
    ) -> BinaryHeap<EarliestFirst> {
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

    // pub fn run_isolated_join_layer(&mut self, match_instances: &mut Vec<(u32, MatchInstance<'p>)>) {
    pub fn run_isolated_join_layer(&mut self, match_instances: &mut Vec<(u32, MatchInstance)>) {
        let num_pat_event = self.pattern.events.len();
        for (sub_pattern_id, match_instance) in match_instances.drain(0..) {
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
        debug!("full matches: {:#?}", self.full_match.len());
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
    P: Iterator<Item = (u32, composition_layer::MatchInstance)>,
    // P: Iterator<Item = (u32, composition_layer::MatchInstance<'p>)>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        let num_pat_event = self.pattern.events.len();
        while self.full_match.is_empty() {
            let (sub_pattern_id, match_instance) = self.prev_layer.next()?;

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
pub mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::input_event::InputEvent;
    use crate::match_event::{MatchEvent, RawEvents};
    use crate::pattern::decompose;
    use crate::{
        pattern::{parser::parse_json, SubPattern},
        process_layers::{composition_layer::MatchInstance, JoinLayer},
        universal_match_event::UniversalMatchEvent,
    };
    use itertools::{enumerate, Itertools};
    use log::debug;
    use nix::libc::input_event;
    use serde_json::Value;
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

    /*
       Note:
           The codes here are duplicate to those in benches/join_layer_benchmark.rs,
           since I haven't think of a good way to use the same piece of code to simultanenouly
           perform testing and benchmarking.

    */
    // fn gen_match_instance_from_subpattern<'p>(sub_pattern: &SubPattern<'p>, set_time: u64) -> MatchInstance<'p> {
    fn gen_match_instance_from_subpattern<'p>(
        sub_pattern: &SubPattern<'p>,
        set_time: u64,
    ) -> MatchInstance {
        let mut match_events = vec![];
        let mut match_entities = vec![];
        for match_event in &sub_pattern.events {
            // match_events.push(UniversalMatchEvent {
            //     matched: *match_event,
            //     start_time: set_time,
            //     end_time: set_time,
            //     subject_id: match_event.subject.id as u64,
            //     object_id: match_event.object.id as u64,
            //     event_ids: vec![match_event.id as u64].into_boxed_slice(),
            // });

            let input_event = InputEvent::new(
                set_time,
                match_event.id as u64,
                &match_event.signature,
                match_event.subject.id as u64,
                &match_event.subject.signature,
                match_event.object.id as u64,
                &match_event.object.signature,
            );

            match_events.push(MatchEvent {
                // matched: *match_event,
                // start_time: set_time,
                // end_time: set_time,
                match_id: match_event.id as u32,
                subject_id: match_event.subject.id as u64,
                object_id: match_event.object.id as u64,
                // event_ids: vec![match_event.id as u64].into_boxed_slice(),
                raw_events: RawEvents::Single(Rc::new(input_event)),
            });

            // We prescribe that the input event id is identical to the pattern event id.
            match_entities.push((match_event.subject.id as u64, match_event.subject.id as u64));
            match_entities.push((match_event.object.id as u64, match_event.object.id as u64));
        }

        match_entities = match_entities.into_iter().sorted().unique().collect_vec();

        let event_ids = match_events
            .iter()
            // .flat_map(|x| x.event_ids.iter())
            .flat_map(|x| x.raw_events.get_ids())
            // .cloned()
            .sorted()
            .collect_vec();

        // Create match instances for each subpattern.
        MatchInstance {
            start_time: set_time,
            match_events: match_events.into_boxed_slice(),
            match_entities: match_entities.into_boxed_slice(),
            event_ids: event_ids.into_boxed_slice(),
            state_id: 0,
        }
    }

    fn gen_match_instances<'p>(
        sub_patterns: &Vec<SubPattern<'p>>,
        has_id: &[usize],
        set_time: u64,
    ) -> Vec<(u32, MatchInstance)> {
    // ) -> Vec<(u32, MatchInstance<'p>)> {
        let mut match_instances = vec![];
        for (id, sub_pattern) in enumerate(sub_patterns) {
            if has_id.binary_search(&id).is_err() {
                continue;
            }
            match_instances.push((
                sub_pattern.id as u32,
                gen_match_instance_from_subpattern(sub_pattern, set_time),
            ));
        }

        match_instances
    }

    #[test_log::test]
    fn run_join_layer() {
        let raw_pattern = r#"{"Version": "0.2.0", "UseRegex": true, "Entities": [{"ID": 0, "Signature": "0"}, {"ID": 1, "Signature": "1"}, {"ID": 2, "Signature": "2"}, {"ID": 3, "Signature": "3"}, {"ID": 4, "Signature": "4"}, {"ID": 5, "Signature": "5"}, {"ID": 6, "Signature": "6"}, {"ID": 7, "Signature": "7"}, {"ID": 8, "Signature": "8"}], "Events": [{"ID": 0, "Signature": "0", "SubjectID": 0, "ObjectID": 1, "Parents": []}, {"ID": 1, "Signature": "1", "SubjectID": 3, "ObjectID": 4, "Parents": [0]}, {"ID": 2, "Signature": "2", "SubjectID": 7, "ObjectID": 8, "Parents": [0]}, {"ID": 3, "Signature": "3", "SubjectID": 5, "ObjectID": 2, "Parents": [0]}, {"ID": 4, "Signature": "4", "SubjectID": 4, "ObjectID": 5, "Parents": [1]}, {"ID": 5, "Signature": "5", "SubjectID": 2, "ObjectID": 6, "Parents": [3]}, {"ID": 6, "Signature": "6", "SubjectID": 5, "ObjectID": 7, "Parents": [2]}, {"ID": 7, "Signature": "7", "SubjectID": 1, "ObjectID": 2, "Parents": [0]}]}"#;
        let json_obj: Value = serde_json::from_str(raw_pattern).expect("error reading json");
        let pattern = parse_json(&json_obj).expect("Failed to parse pattern");

        let windows_size = 1 * 1000;
        let sub_patterns = decompose(&pattern);

        debug!("sub_patterns: {:#?}", sub_patterns);

        let mut join_layer = JoinLayer::new((), &pattern, &sub_patterns, windows_size);

        /*
            Buffer structure:
                      0
                   /     \
                  1       2
                /   \   /   \
               3     4 5     6
            Below shows the expected number of joins occur in each buffer node, with format "(node1, node2): <all, success>"
                (3, 4): <8, 2>
                (5, 6): <4, 4>
                (1, 2): <2, 1>

            Expected complete pattern match: 1
            Rate of success joins: 50% (fail reason: order relation)

            Note that the above figures are for a single iteration. There are 100 iterations, and thus all numbers should
            be multiplied by 100.

        */
        let mut match_instances = vec![];
        // Mind that the end-of-loop "0" and "1" instances may be joined with beginning-of-loop "[0, 1]" instances,
        // if timestamps are not properly set.
        for i in 0..100 {
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[0, 1],
                9 * i * windows_size + 100,
            ));
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[2, 3],
                (9 * i + 1) * windows_size + 101,
            ));
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[1, 3],
                (9 * i + 2) * windows_size + 1,
            ));
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[0, 2],
                (9 * i + 3) * windows_size,
            )); // subpattern 0 and 1 join fail
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[1],
                (9 * i + 3) * windows_size + 1,
            )); // subpattern 0 and 1 join success
            match_instances.append(&mut gen_match_instances(
                &sub_patterns,
                &[3],
                (9 * i + 3) * windows_size + 2,
            )); // subpattern 0 and 1 join success

            for j in 0..5 {
                match_instances.append(&mut gen_match_instances(
                    &sub_patterns,
                    &[1],
                    (9 * i + j + 4) * windows_size + 3 + 2 * j,
                ));
                match_instances.append(&mut gen_match_instances(
                    &sub_patterns,
                    &[0],
                    (9 * i + j + 4) * windows_size + 4 + 2 * j,
                ));
                // match_instances.append(&mut gen_match_instances(&sub_patterns, &[2, 3], (9*i+j+4)*windows_size + 3 + 2*j));
                // match_instances.append(&mut gen_match_instances(&sub_patterns, &[2, 3], (9*i+j+4)*windows_size + 4 + 2*j));
            }
        }

        // // Randomly shuffle match_instances
        // let seed = 123456;
        // let mut rng = ChaChaRng::seed_from_u64(seed);
        // if !fixed {
        //     rng = ChaChaRng::from_entropy();
        // }
        // match_instances.shuffle(&mut rng);

        join_layer.run_isolated_join_layer(&mut match_instances);
    }
}
