mod sub_pattern_buffer;
mod sub_pattern_match;

use crate::pattern::Pattern;
use crate::pattern::SubPattern;
use crate::pattern_match::PatternMatch;
use log::debug;
use std::cmp::max;
use std::cmp::min;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::vec;
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
    sub_pattern_buffers: Vec<SubPatternBuffer>,

    /// See `clear_expired()`.
    window_size: u64,

    /// Complete pattern matches.
    full_match: Vec<PatternMatch>,

    /// The sibling buffer of buffer `x` is `sibling_id_map[x]`.
    sibling_id_map: Vec<usize>,
    /// The parent buffer of buffer `x` is `parent_id_map[x]`.
    parent_id_map: Vec<usize>,
}

impl<'p, P> JoinLayer<'p, P> {
    /// Given two existing sub-pattern buffers, group them as a pair, generate relations between
    /// them, and merge them to create their parent buffer.
    fn create_buffer_pair(
        buffer_id1: usize,
        buffer_id2: usize,
        new_buffer_id: usize,
        pattern: &'p Pattern,
        sub_pattern_buffers: &mut Vec<SubPatternBuffer>,
    ) {
        let relations = SubPatternBuffer::generate_relations(
            pattern,
            &sub_pattern_buffers[buffer_id1],
            &sub_pattern_buffers[buffer_id2],
        );

        // update relations
        sub_pattern_buffers[buffer_id1].relation = relations.clone();
        sub_pattern_buffers[buffer_id2].relation = relations;

        sub_pattern_buffers.push(SubPatternBuffer::merge_buffers(
            &sub_pattern_buffers[buffer_id1],
            &sub_pattern_buffers[buffer_id2],
            new_buffer_id,
        ));
    }

    /// Mainly construct the tree-structure of sub-pattern buffers.
    /// Note that all sub-pattern buffers have shared-node relation with their corresponding sibling.
    pub fn new(
        prev_layer: P,
        pattern: &'p Pattern,
        sub_patterns: &[SubPattern<'p>],
        window_size: u64,
    ) -> Self {
        let buffer_len = 2 * sub_patterns.len() - 1;
        let mut sub_pattern_buffers = Vec::with_capacity(buffer_len);
        let mut sibling_id_map = vec![0usize; buffer_len];
        let mut parent_id_map = vec![0usize; buffer_len];

        // Initial buffers for decomposed sub-patterns.
        for (i, sub_pattern) in sub_patterns.iter().enumerate() {
            let buffer_id = i;
            sub_pattern_buffers.push(SubPatternBuffer::new(
                buffer_id,
                sub_pattern,
                pattern.entities.len(),
                pattern.events.len(),
            ));
        }

        let mut union_find = UnionFind::new(buffer_len);

        // Indicate whether a sub-pattern buffer has been processed or not.
        let mut merged = vec![false; buffer_len];

        // For `(h, i, j)` in `min_heap`, buffers `i` and `j` have shared-node relation (can be merged).
        // If they are merged, the resulting buffer height would be `h`.
        let mut min_heap = BinaryHeap::new();
        let shared_node_lists = Self::gen_shared_node_lists(sub_patterns);
        for (i, list) in shared_node_lists.iter().enumerate() {
            for j in list {
                // avoid duplicates
                if *j <= i {
                    continue;
                }
                min_heap.push(Reverse((2u32, i, *j)));
            }
        }

        // Each time pop the can-be-merged buffer pair with minimal resulting height.
        while let Some(Reverse((height, i, j))) = min_heap.pop() {
            if merged[i] || merged[j] {
                continue;
            }

            merged[i] = true;
            merged[j] = true;

            let new_buffer_id = sub_pattern_buffers.len();
            Self::create_buffer_pair(i, j, new_buffer_id, pattern, &mut sub_pattern_buffers);
            sibling_id_map[i] = j;
            sibling_id_map[j] = i;
            parent_id_map[i] = new_buffer_id;
            parent_id_map[j] = new_buffer_id;

            debug!(
                "buffer {} and buffer {} are merged into buffer {}",
                i, j, new_buffer_id
            );

            union_find.merge(i, j, new_buffer_id);

            let mut visited = HashSet::new();
            visited.insert(new_buffer_id);

            // Find all buffers that has shared-node relation with the newly created buffer, for futher merger.
            for k in 0..sub_patterns.len() {
                let cur_root = union_find.get_root(k);
                if visited.contains(&cur_root) {
                    continue;
                }

                for id in &shared_node_lists[k] {
                    // has shared node relation
                    if union_find.get_root(*id) == new_buffer_id {
                        let new_height = max(height, union_find.get_height(cur_root)) + 1;
                        min_heap.push(Reverse((new_height, new_buffer_id, cur_root)));

                        visited.insert(cur_root);
                        break;
                    }
                }
            }
        }

        Self {
            prev_layer,
            pattern,
            sub_pattern_buffers,
            window_size,
            full_match: Vec::new(),
            sibling_id_map,
            parent_id_map,
        }
    }

    /// For each sub-pattern, calculate the sub-patterns that have shared-node relation with itself.
    fn gen_shared_node_lists(sub_patterns: &[SubPattern<'p>]) -> Vec<Vec<usize>> {
        let mut shared_node_lists = vec![Vec::new(); sub_patterns.len()];
        for (i, sub_pattern1) in sub_patterns.iter().enumerate() {
            let entity_ids1: HashSet<usize> = sub_pattern1
                .events
                .iter()
                .flat_map(|e| [e.subject.id, e.object.id])
                .collect();
            for (j, sub_pattern2) in sub_patterns.iter().enumerate() {
                if j <= i {
                    continue;
                }

                let entity_ids2: HashSet<usize> = sub_pattern2
                    .events
                    .iter()
                    .flat_map(|e| [e.subject.id, e.object.id])
                    .collect();

                if Self::has_shared_node(&entity_ids1, &entity_ids2) {
                    shared_node_lists[i].push(j);
                    shared_node_lists[j].push(i);
                }
            }
        }
        shared_node_lists
    }

    /// Check whether two entity (node) lists have any shared element.
    fn has_shared_node(entity_ids1: &HashSet<usize>, entity_ids2: &HashSet<usize>) -> bool {
        return entity_ids1.intersection(&entity_ids2).next() != None;
    }

    /// Convert `SubPatternMatch to `PatternMatch`.
    fn pattern_match_conversion(buffer: &mut BinaryHeap<EarliestFirst>) -> Vec<PatternMatch> {
        buffer
            .drain()
            .map(|sub_pattern_match| sub_pattern_match.0.into())
            .collect()
    }

    /// Add the sub-pattern matches in the root buffer to the final buffer.
    ///
    /// The uniqueness of matches is handled in the next layer (The Uniqueness Layer).
    fn add_to_answer(&mut self) {
        let root_id = self.get_root_buffer_id();
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
    fn join_with_sibling(&mut self, my_id: usize, sibling_id: usize) -> BinaryHeap<EarliestFirst> {
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
            if buffer_id == self.get_root_buffer_id() {
                self.add_to_answer();
                break;
            }

            // Clear only the sibling buffer, since we can clear the current buffer when needed (deferred).
            self.clear_expired(current_time, self.get_sibling_id(buffer_id));

            let joined = self.join_with_sibling(buffer_id, self.get_sibling_id(buffer_id));
            let parent_id = self.get_parent_id(buffer_id);

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
                let buffer_id = get_buffer_id(sub_match.id);
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

    fn get_root_buffer_id(&self) -> usize {
        self.sub_pattern_buffers.len() - 1
    }

    fn get_sibling_id(&self, id: usize) -> usize {
        self.sibling_id_map[id]
    }

    fn get_parent_id(&self, id: usize) -> usize {
        self.parent_id_map[id]
    }
}

/// Get the corresponding buffer id of a sub-pattern match.
fn get_buffer_id(sub_match_id: usize) -> usize {
    sub_match_id
}

impl<'p, P> Iterator for JoinLayer<'p, P>
where
    P: Iterator<Item = (u32, composition_layer::MatchInstance)>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        let num_pat_event = self.pattern.events.len();
        while self.full_match.is_empty() {
            let (sub_pattern_id, match_instance) = self.prev_layer.next()?;

            if let Some(sub_match) =
                SubPatternMatch::build(sub_pattern_id, match_instance, num_pat_event)
            {
                // Note that `sub_match_id` should be identical as `sub_pattern_id`
                let buffer_id = get_buffer_id(sub_match.id);
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

/// Union-find tree, a.k.a disjoint set
struct UnionFind {
    /// Store the root (representative element) of a union-find tree (disjoint set).
    /// For element `id`, if `roots[id] < 0`, them `id` is the root of the tree it belongs to.
    /// Otherwise, `roots[id]` is the parent of `id` (but not necessary the root).
    ///
    /// A disjoint set corresponds to a sub-pattern buffer in the Join layer.
    /// Note that for `roots[id] < 0`, the value `abs(roots[id])` corresponds to the
    /// *height of the buffer* in the sub-pattern buffer tree structure.
    roots: Vec<i64>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            roots: vec![-1; size], // height
        }
    }

    /// Return the root element for `id`.
    fn get_root(&mut self, id: usize) -> usize {
        if self.roots[id] < 0 {
            id
        } else {
            self.roots[id] = self.get_root(self.roots[id] as usize) as i64;
            self.roots[id] as usize
        }
    }

    /// Return the buffer height that `id` belongs to.
    fn get_height(&self, id: usize) -> u32 {
        -self.roots[id] as u32
    }

    /// Merge two union-find trees, and create a node (corresponds to a new buffer) as the new root.
    fn merge(&mut self, id1: usize, id2: usize, new_root: usize) {
        let root1 = self.get_root(id1);
        let root2 = self.get_root(id2);
        if root1 == root2 {
            return;
        }

        // A new node (corresponds to a new buffer)
        self.roots[new_root] = min(self.roots[root1], self.roots[root2]) - 1;
        self.roots[root1] = new_root as i64;
        self.roots[root2] = new_root as i64;
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
    };
    use itertools::{enumerate, Itertools};
    use log::debug;
    use serde_json::Value;
    #[test]
    fn test_generate_sub_pattern_buffers() {
        let pattern = Pattern::parse("../data/universal_patterns/SP8_regex.json")
            .expect("Failed to parse pattern");

        let window_size = 1800 * 1000;
        let sub_patterns = decompose(&pattern);
        println!("{:#?}", sub_patterns);
        println!("\n\n");
        let join_layer = JoinLayer::new((), &pattern, &sub_patterns, window_size);
        println!("{:#?}", join_layer.sub_pattern_buffers);
    }

    #[test_log::test]
    fn test_sub_pattern_buffer_shared_node_relation() {
        let pattern = Pattern::parse("../data/universal_patterns/SP6_regex.json")
            .expect("Failed to parse pattern");

        let window_size = 1800 * 1000;
        let sub_patterns = decompose(&pattern);
        let join_layer = JoinLayer::new((), &pattern, &sub_patterns, window_size);

        debug!(
            "sub_pattern_buffer len: {}",
            join_layer.sub_pattern_buffers.len()
        );
        debug!("num of sub_pattern: {}", sub_patterns.len());
        debug!("sibling id map len: {}", join_layer.sibling_id_map.len());

        for sub_pattern_buffer in &join_layer.sub_pattern_buffers {
            let sibling_id = join_layer.get_sibling_id(sub_pattern_buffer.id);
            let sibling_buffer = &join_layer.sub_pattern_buffers[sibling_id];

            assert_ne!(
                sub_pattern_buffer
                    .node_id_list
                    .intersection(&sibling_buffer.node_id_list)
                    .next(),
                None
            );
        }
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
                match_id: match_event.id as u32,
                input_subject_id: match_event.subject.id as u64,
                input_object_id: match_event.object.id as u64,
                pattern_subject_id: match_event.subject.id as u64,
                pattern_object_id: match_event.object.id as u64,
                raw_events: RawEvents::Single(Rc::new(input_event)),
            });

            // We prescribe that the input event id is identical to the pattern event id.
            match_entities.push((match_event.subject.id as u64, match_event.subject.id as u64));
            match_entities.push((match_event.object.id as u64, match_event.object.id as u64));
        }

        match_entities = match_entities.into_iter().sorted().unique().collect_vec();

        let event_ids = match_events
            .iter()
            .flat_map(|x| x.raw_events.get_ids())
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
            }
        }

        join_layer.run_isolated_join_layer(&mut match_instances);
    }
}
