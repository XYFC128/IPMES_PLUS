mod entity_encode;
mod filter;
mod flow_runner;
mod flow_tracer;
mod instance_runner;
mod instance_storage;
pub mod match_instance;
mod pattern_info;
mod state;
mod state_table;

pub use match_instance::{InputEntityId, PatternEntityId};

use crate::input_event::InputEvent;
use crate::pattern::{PatternEventType, SubPattern};
use ahash::HashMap;
use flow_runner::FlowRunner;
use instance_runner::InstanceRunner;
use instance_storage::InstanceStorage;
use log::debug;
pub use match_instance::MatchInstance;
use pattern_info::{FlowPattern, FreqPattern, PatternInfo, SinglePattern};
use regex::Error as RegexError;
use state::*;
use state_table::StateTable;
use std::rc::Rc;

pub struct CompositionLayer<'p, P> {
    prev_layer: P,
    window_size: u64,
    cur_time: u64,
    pattern_infos: Vec<PatternInfo<'p>>,
    storage: InstanceStorage,
    // storage: InstanceStorage<'p>,
    runner: InstanceRunner,
    flow_runner: FlowRunner,
    state_table: StateTable,
}

impl<'p, P> CompositionLayer<'p, P> {
    pub fn new(
        prev_layer: P,
        decomposition: &[SubPattern<'p>],
        window_size: u64,
        use_regex: bool,
    ) -> Result<Self, RegexError> {
        let state_table = StateTable::new(decomposition);
        let storage = InstanceStorage::init_from_state_table(&state_table);
        let runner = InstanceRunner::new(decomposition, window_size, use_regex)?;
        let (flow_runner, sig_indices) = FlowRunner::new(decomposition, window_size, use_regex)?;
        let pattern_infos = Self::build_pattern_infos(decomposition, &sig_indices, &state_table);

        Ok(Self {
            prev_layer,
            window_size,
            cur_time: 0,
            pattern_infos,
            storage,
            runner,
            flow_runner,
            state_table,
        })
    }

    /// build pattern_infos
    ///
    /// Arguments:
    /// - `decomposition`: the decomposition of the pattern
    /// - `sig_indices`: maps the entity id to the signature index in the `FlowRunner`
    /// - `state_table`: state table
    pub fn build_pattern_infos(
        decomposition: &[SubPattern<'p>],
        sig_indices: &HashMap<usize, usize>,
        state_table: &StateTable,
    ) -> Vec<PatternInfo<'p>> {
        let mut match_idx = 0usize;
        let mut signature_idx = 0usize;
        let mut pattern_infos = vec![];
        for sub_pattern in decomposition {
            for pattern in &sub_pattern.events {
                use PatternEventType::*;
                let shared_node_info = state_table.get_shared_node_info(match_idx);
                let info: PatternInfo = match pattern.event_type {
                    Default => SinglePattern {
                        pattern,
                        match_idx,
                        shared_node_info,
                        signature_idx,
                    }
                    .into(),

                    Frequency(frequency) => FreqPattern {
                        pattern,
                        match_idx,
                        shared_node_info,
                        signature_idx,
                        frequency,
                    }
                    .into(),

                    Flow => {
                        let src_sig_idx = *sig_indices.get(&pattern.subject.id).unwrap();
                        let dst_sig_idx = *sig_indices.get(&pattern.object.id).unwrap();
                        FlowPattern {
                            pattern,
                            match_idx,
                            shared_node_info,
                            src_sig_idx,
                            dst_sig_idx,
                        }
                        .into()
                    }
                };
                pattern_infos.push(info);
                match_idx += 1;
                if !matches!(pattern.event_type, Flow) {
                    signature_idx += 1;
                }
            }
        }
        pattern_infos
    }

    pub fn add_batch(&mut self, batch: &[Rc<InputEvent>]) {
        let time = if let Some(first) = batch.first() {
            first.timestamp
        } else {
            return;
        };

        self.cur_time = time;
        self.runner.set_batch(batch, time);
        self.flow_runner.set_batch(batch, time);

        // TODO: Consider active windowing
    }

    pub fn advance(&mut self) {
        for info in &self.pattern_infos {
            match info {
                PatternInfo::Single(info) => {
                    self.runner.run(info, &mut self.storage, &self.state_table)
                }
                PatternInfo::Freq(info) => {
                    self.runner
                        .run_freq(info, &mut self.storage, &self.state_table)
                }
                PatternInfo::Flow(info) => {
                    self.flow_runner
                        .run(info, &mut self.storage, &self.state_table)
                }
            }
        }
    }
}

impl<'p, P> Iterator for CompositionLayer<'p, P>
where
    P: Iterator<Item = Box<[Rc<InputEvent>]>>,
{
    type Item = (u32, MatchInstance);

    fn next(&mut self) -> Option<Self::Item> {
        while self.storage.output_instances.is_empty() {
            let batch = self.prev_layer.next()?;
            self.add_batch(&batch);
            self.advance();
        }

        if let Some(output) = self.storage.output_instances.last() {
            debug!("output: ({:?})", output);
        }
        self.storage.output_instances.pop()
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::rc::Rc;

    use itertools::Itertools;

    use super::*;
    use crate::input_event::InputEvent;
    use crate::match_event::MatchEvent;
    use crate::pattern::{Pattern, PatternEventType};
    use crate::universal_match_event::UniversalMatchEvent;

    /// Creates a pattern consists of 3 event and 4 entities. They form a path from v0 to v3.
    ///
    /// ```text
    ///     e0       e1       e2
    /// v0 ----> v1 ----> v2 ----> v3
    /// ```
    fn basic_pattern() -> Pattern {
        Pattern::from_graph(
            &["v0", "v1", "v2", "v3"],
            &[(0, 1, "e0"), (1, 2, "e1"), (2, 3, "e2")],
            false,
        )
    }

    /// Creates a batch containing only one input event.
    ///
    /// Parameters:
    /// - `eid`: the id of the input event, also used as the timestamp of the event
    /// - `sub_id`: the id of the subject
    /// - `obj_id`: the id of the object
    /// - `sig`: signature of the event, the subject and the object separated by '#'
    fn event(eid: u64, sub_id: u64, obj_id: u64, sig: &str) -> Box<[Rc<InputEvent>]> {
        let sigs: Vec<&str> = sig.split('#').collect();
        vec![Rc::new(InputEvent::new(
            eid, eid, sigs[0], sub_id, sigs[1], obj_id, sigs[2],
        ))]
        .into_boxed_slice()
    }

    fn verify_instance(
        result: Option<(u32, MatchInstance)>,
        expect_sub_pattern_id: u32,
        expect_ts: u64,
        expect_ids: &[u64],
    ) {
        if let Some(output) = result {
            assert_eq!(output.0, expect_sub_pattern_id, "sub_pattern_id");

            let instance = output.1;
            assert_eq!(instance.start_time, expect_ts, "start_time");

            let mut sorted_ids = expect_ids.to_vec();
            sorted_ids.sort_unstable();
            assert_eq!(*instance.event_ids, sorted_ids, "event_ids");

            let ids: Vec<u64> = instance
                .match_events
                .iter()
                .map(|x| x.raw_events.get_ids().collect_vec()[0])
                // .map(|x| x.event_ids[0])
                .collect();
            assert_eq!(ids, *expect_ids, "match_events");
        } else {
            panic!("result is None");
        }
    }

    #[test]
    fn test_basic() {
        let pattern = basic_pattern();
        let window_size = u64::MAX;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#v2"),
            event(2, 2, 3, "e2#v2#v3"),
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        verify_instance(
            layer.next(),
            0,          // sub-pattern id
            0,          // start_time
            &[0, 1, 2], // matched input event ids
        );
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_event_uniqueness() {
        let pattern = Pattern::from_graph(
            &["v0", "v1", "v2"],
            &[(0, 1, "e0"), (1, 2, "e1"), (1, 2, "e.")],
            true,
        );
        let window_size = u64::MAX;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#v2"), // matches pattern event 1 & 2
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        assert!(layer.next().is_none());
    }

    #[test]
    fn test_entity_uniqueness() {
        let pattern = Pattern::from_graph(
            &["v0", "v1", "v.", "v3"],
            &[(0, 1, "e0"), (1, 2, "e1"), (1, 3, "e2")],
            true,
        );
        let window_size = u64::MAX;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#v3"),
            event(2, 1, 2, "e2#v1#v3"), // input entity 2 duplicates
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        assert!(layer.next().is_none());
    }

    #[test]
    fn test_basic_windowing() {
        let pattern = basic_pattern();
        let window_size = 3;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#v2"),
            event(4, 2, 3, "e2#v2#v3"),
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        assert!(layer.next().is_none());
    }

    // fn verify_event(
    //     match_event: &UniversalMatchEvent,
    //     time_pair: (u64, u64),
    //     entity_id_pair: (u64, u64),
    //     event_ids: &[u64],
    // ) {
    //     assert_eq!(match_event.start_time, time_pair.0);
    //     assert_eq!(match_event.end_time, time_pair.1);
    //     assert_eq!(match_event.subject_id, entity_id_pair.0);
    //     assert_eq!(match_event.object_id, entity_id_pair.1);
    //     assert_eq!(*match_event.event_ids, *event_ids)
    // }

    fn verify_event(
        match_event: &MatchEvent,
        time_pair: (u64, u64),
        entity_id_pair: (u64, u64),
        event_ids: &[u64],
    ) {
        let (start_time, end_time) = match_event.raw_events.get_interval();
        assert_eq!(start_time, time_pair.0);
        assert_eq!(end_time, time_pair.1);
        assert_eq!(match_event.input_subject_id, entity_id_pair.0);
        assert_eq!(match_event.input_object_id, entity_id_pair.1);
        assert_eq!(*match_event.raw_events.get_ids().collect_vec(), *event_ids)
    }

    #[test]
    fn test_frequency() {
        let mut pattern = basic_pattern();
        pattern.events[1].event_type = PatternEventType::Frequency(3);
        let window_size = u64::MAX;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#v2"),
            event(2, 1, 2, "e1#v1#v2"),
            event(3, 1, 2, "e1#v1#v2"),
            event(4, 2, 3, "e2#v2#v3"),
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        let match_events = layer.next().unwrap().1.match_events;
        verify_event(&match_events[0], (0, 0), (0, 1), &[0]);
        verify_event(&match_events[1], (1, 3), (1, 2), &[1, 2, 3]);
        verify_event(&match_events[2], (4, 4), (2, 3), &[4]);
        assert!(layer.next().is_none());
    }

    #[test]
    fn test_flow() {
        let mut pattern = basic_pattern();
        pattern.events[1].event_type = PatternEventType::Flow;
        let window_size = u64::MAX;
        let decomposition = [SubPattern {
            id: 0,
            events: pattern.events.iter().collect(),
        }];

        let input = [
            event(0, 0, 1, "e0#v0#v1"),
            event(1, 1, 2, "e1#v1#vx"),
            event(2, 2, 3, "e1#vx#vy"),
            event(3, 3, 4, "e1#vy#v2"),
            event(4, 4, 5, "e2#v2#v3"),
        ];
        let mut layer =
            CompositionLayer::new(input.into_iter(), &decomposition, window_size, false).unwrap();

        let match_events = layer.next().unwrap().1.match_events;
        verify_event(&match_events[0], (0, 0), (0, 1), &[0]);
        verify_event(&match_events[1], (1, 3), (1, 4), &[]);
        verify_event(&match_events[2], (4, 4), (4, 5), &[4]);
        assert!(layer.next().is_none());
    }
}
