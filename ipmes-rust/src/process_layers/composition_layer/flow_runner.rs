use super::flow_tracer::FlowTracer;
use super::instance_storage::{InstanceStorage, StorageRequest};
use super::pattern_info::FlowPattern;
use super::state_table::StateTable;
use crate::input_event::InputEvent;
use crate::pattern::{PatternEntity, PatternEvent, PatternEventType, SubPattern};
use crate::universal_match_event::UniversalMatchEvent;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use regex::{Error, RegexSet, SetMatches};
use std::collections::hash_map::Entry;
use std::rc::Rc;

struct NodeMatchResult {
    update_time: u64,
    set_matches: SetMatches,
}

pub struct FlowRunner {
    flow_tracer: FlowTracer<u64>,

    /// the modified destination nodes after last call to set_batch
    modified: HashSet<u64>,

    node_regexes: RegexSet,

    node_match_results: HashMap<u64, NodeMatchResult>,

    /// the length of the time window
    window_size: u64,

    /// the time of the latest batch
    cur_time: u64,
}

impl FlowRunner {
    pub fn new(
        decomposition: &[SubPattern],
        window_size: u64,
        use_regex: bool,
    ) -> Result<(Self, HashMap<usize, usize>), Error> {
        let mut sig_indices = HashMap::new();
        let mut regex_patterns = vec![];
        let mut add_regex_pattern = |ent: &PatternEntity| {
            if let Entry::Vacant(e) = sig_indices.entry(ent.id) {
                e.insert(regex_patterns.len());
                if !use_regex {
                    regex_patterns.push(format!("^{}$", regex::escape(&ent.signature)));
                } else {
                    regex_patterns.push(format!("^{}$", &ent.signature));
                }
            }
        };

        for sub_pattern in decomposition {
            for pattern in &sub_pattern.events {
                use PatternEventType::Flow;
                if !matches!(pattern.event_type, Flow) {
                    continue;
                }

                add_regex_pattern(&pattern.subject);
                add_regex_pattern(&pattern.object);
            }
        }

        let node_regexes = RegexSet::new(regex_patterns)?;
        let flow_tracer = FlowTracer::new(window_size);

        Ok((
            Self {
                flow_tracer,
                modified: HashSet::new(),
                node_regexes,
                node_match_results: HashMap::new(),
                window_size,
                cur_time: 0,
            },
            sig_indices,
        ))
    }

    pub fn set_batch(&mut self, batch: &[Rc<InputEvent>], time: u64) {
        for event in batch {
            self.update_node_match(event.subject_id, event.get_subject_signature());
            self.update_node_match(event.object_id, event.get_object_signature());
        }

        let iter = batch
            .iter()
            .map(|event| (event.subject_id, event.object_id, event.event_id));
        self.modified = self.flow_tracer.add_batch(iter, time);
        self.cur_time = time;
    }

    fn update_node_match(&mut self, node_id: u64, signature: &str) {
        self.node_match_results
            .entry(node_id)
            .and_modify(|ent| ent.update_time = self.cur_time)
            .or_insert(NodeMatchResult {
                update_time: self.cur_time,
                set_matches: self.node_regexes.matches(signature),
            });
    }

    fn is_node_match(&self, id: u64, sig_idx: usize) -> bool {
        if let Some(res) = self.node_match_results.get(&id) {
            res.set_matches.matched(sig_idx)
        } else {
            false
        }
    }

    pub fn run<'p>(
        &self,
        info: &FlowPattern<'p>,
        storage: &mut InstanceStorage<'p>,
        state_table: &StateTable,
    ) {
        let window_bound = self.cur_time.saturating_sub(self.window_size);

        let mut new_instances = vec![];
        for dst in &self.modified {
            if !self.is_node_match(*dst, info.dst_sig_idx) {
                continue;
            }

            let match_idx = info.match_idx;
            let reach_set = self.flow_tracer.get_reachset(*dst).unwrap();
            for src in reach_set.get_updated_nodes() {
                if !self.is_node_match(*src, info.src_sig_idx) {
                    continue;
                }

                let request = StorageRequest {
                    match_idx,
                    subject_id: *src,
                    object_id: *dst,
                    shared_node_info: info.shared_node_info,
                };

                let flow = self
                    .get_flow(*src, *dst, info.pattern)
                    .expect("(src, dst) forms a valid flow");
                for instance in storage.query_with_windowing(&request, window_bound) {
                    if let Some(mut new_instance) =
                        instance.clone_extend_flow(flow.clone(), info.shared_node_info)
                    {
                        new_instance.state_id = state_table.get_next_state(instance.state_id);
                        new_instances.push(new_instance);
                    }
                }
            }
        }
        storage.store_new_instances(new_instances.into_iter(), state_table);
    }

    pub fn get_flow<'p>(
        &self,
        src: u64,
        dst: u64,
        matched: &'p PatternEvent,
    ) -> Option<UniversalMatchEvent<'p>> {
        let mut path = Vec::new();
        self.flow_tracer.visit_path(src, dst, |eid| path.push(eid));
        if path.is_empty() {
            return None;
        }

        let start_time = self.flow_tracer.get_update_time(src, dst).unwrap();
        Some(UniversalMatchEvent {
            matched,
            start_time,
            end_time: self.cur_time,
            subject_id: src,
            object_id: dst,
            event_ids: path.into_boxed_slice(),
        })
    }
}
