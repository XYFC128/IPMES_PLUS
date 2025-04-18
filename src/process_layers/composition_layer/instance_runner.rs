use super::instance_storage::StorageRequest;
use super::match_instance::FreqInstance;
use super::pattern_info::{FreqPattern, SharedNodeInfo, SinglePattern};
use super::state_table::StateTable;
use super::{InstanceStorage, MatchInstance};
use crate::input_event::InputEvent;
use crate::match_event::{MatchEvent, RawEvents};
use crate::pattern::{PatternEvent, PatternEventType, SubPattern};
use regex::{Error, RegexSet, SetMatches};
use std::rc::Rc;

pub struct InstanceRunner {
    window_size: u64,
    /// A set of all event and entity signatures of a given pattern.
    event_regexes: RegexSet,
    cur_time: u64,
    cur_batch: Vec<(Rc<InputEvent>, SetMatches)>,
}

/// Construct a regex pattern (signature) from a pattern event. The construction is identical to 
/// that of input events. See `InputEvent.get_signatures()`.
fn construct_regex_pattern(pattern: &PatternEvent, escape_regex: bool) -> String {
    let regex_pattern = format!(
        "{}\0{}\0{}",
        pattern.signature, pattern.subject.signature, pattern.object.signature
    );

    if escape_regex {
        format!("^{}$", regex::escape(&regex_pattern))
    } else {
        format!("^{}$", regex_pattern)
    }
}

impl InstanceRunner {
    pub fn new(
        decomposition: &[SubPattern],
        window_size: u64,
        use_regex: bool,
    ) -> Result<Self, Error> {
        let mut patterns = vec![];
        for sub_pattern in decomposition {
            for pattern in &sub_pattern.events {
                use PatternEventType::*;
                if matches!(pattern.event_type, Flow) {
                    continue;
                }
                patterns.push(construct_regex_pattern(pattern, !use_regex));
            }
        }
        let event_regexes = RegexSet::new(patterns)?;
        Ok(Self {
            window_size,
            event_regexes,
            cur_time: 0,
            cur_batch: vec![],
        })
    }

    /// Match the input batch of events against all pattern events (in terms of signatures). 
    pub fn set_batch(&mut self, batch: &[Rc<InputEvent>], time: u64) {
        self.cur_time = time;
        self.cur_batch.clear();
        for event in batch {
            let result = self.event_regexes.matches(event.get_signatures());
            if result.matched_any() {
                self.cur_batch.push((Rc::clone(event), result));
            }
        }
    }

    /// Execute the composition logic for default-typed pattern event
    pub fn run<'p>(
        &mut self,
        info: &SinglePattern<'p>,
        storage: &mut InstanceStorage,
        state_table: &StateTable,
    ) {
        let window_bound = self.cur_time.saturating_sub(self.window_size);

        let mut new_instances = vec![];
        for (event, sig_match) in &self.cur_batch {
            if !sig_match.matched(info.signature_idx) {
                continue;
            }
            let request = StorageRequest {
                match_idx: info.match_idx,
                subject_id: event.subject_id,
                object_id: event.object_id,
                shared_node_info: info.shared_node_info,
            };
            for instance in storage.query_with_windowing(&request, window_bound) {

                let new_event = MatchEvent {
                    match_id: info.pattern.id as u32,
                    input_subject_id: event.subject_id,
                    input_object_id: event.object_id,
                    pattern_subject_id: info.pattern.subject.id as u64,
                    pattern_object_id: info.pattern.object.id as u64,
                    raw_events: RawEvents::Single(event.clone())
                };
                if let Some(mut new_instance) =
                    instance.clone_extend(new_event, info.shared_node_info)
                {
                    new_instance.state_id = state_table.get_next_state(instance.state_id);
                    new_instances.push(new_instance);
                }
            }
        }
        storage.store_new_instances(new_instances.into_iter(), state_table);
    }

    /// Execute the composition logic for frequency-typed pattern event
    pub fn run_freq<'p>(
        &self,
        info: &FreqPattern<'p>,
        storage: &mut InstanceStorage,
        state_table: &StateTable,
    ) {
        let window_bound = self.cur_time.saturating_sub(self.window_size);

        for (event, sig_match) in &self.cur_batch {
            if !sig_match.matched(info.signature_idx) {
                continue;
            }

            let request = StorageRequest {
                match_idx: info.match_idx,
                subject_id: event.subject_id,
                object_id: event.object_id,
                shared_node_info: info.shared_node_info,
            };

            let mut new_freq_instances = vec![];
            for instance in storage.query_with_windowing(&request, window_bound) {
                if !check_unshared_entity(instance, event, info.shared_node_info) {
                    continue;
                }

                let filter = (info.match_idx, event.subject_id, event.object_id);
                let mut agg_instance =
                    FreqInstance::new(instance.clone(), info.frequency, self.cur_time);
                agg_instance.add_event(event);
                agg_instance.instance.state_id = state_table.get_next_state(instance.state_id);
                new_freq_instances.push((filter, agg_instance));
            }

            let mut new_instances = vec![];
            for instance in storage.query_freq_instances(&request, window_bound) {
                if instance.add_event(event) && instance.is_full() {

                    let raw_events = std::mem::take(&mut instance.new_events);

                    let new_event = MatchEvent {
                        match_id: info.pattern.id as u32,
                        input_subject_id: event.subject_id,
                        input_object_id: event.object_id,
                        pattern_subject_id: info.pattern.subject.id as u64,
                        pattern_object_id: info.pattern.object.id as u64,
                        raw_events: RawEvents::Multiple(raw_events.into_boxed_slice())
                    };

                    if let Some(mut new_instance) = instance
                        .instance
                        .clone_extend(new_event, info.shared_node_info)
                    {
                        new_instance.state_id =
                            state_table.get_next_state(instance.instance.state_id);
                        new_instances.push(new_instance);
                    }
                }
            }
            storage.store_freq_instances(new_freq_instances.into_iter());
            storage.store_new_instances(new_instances.into_iter(), state_table);
        }
    }
}

/// Returns `true` if the unshared entities are not in the [`instance`]
fn check_unshared_entity(
    instance: &MatchInstance,
    event: &InputEvent,
    shared: SharedNodeInfo,
) -> bool {
    use SharedNodeInfo::*;
    match shared {
        None => {
            !instance.contains_eneity(event.subject_id)
                && !instance.contains_eneity(event.object_id)
        }
        Subject => !instance.contains_eneity(event.object_id),
        Object => !instance.contains_eneity(event.subject_id),
        Both => true,
    }
}
