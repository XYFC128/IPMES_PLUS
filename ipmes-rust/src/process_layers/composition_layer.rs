use crate::sub_pattern::SubPattern;

use super::matching_layer::PartialMatchEvent;

mod entity_encode;
mod filter;
mod instance_runner;
mod instance_storage;
mod match_instance;
mod state;
use instance_runner::InstanceRunner;
use instance_storage::InstanceStorage;
use log::debug;
pub use match_instance::MatchInstance;
use state::*;

pub struct CompositionLayer<'p, P> {
    prev_layer: P,
    window_size: u64,
    runner: InstanceRunner<'p>,
    storage: InstanceStorage<'p>,
}

impl<'p, P> CompositionLayer<'p, P> {
    pub fn new(prev_layer: P, decomposition: &[SubPattern<'p>], window_size: u64) -> Self {
        let runner = InstanceRunner::new(decomposition);
        let storage = InstanceStorage::init_from_state_table(&runner.state_table);
        Self {
            prev_layer,
            window_size,
            runner,
            storage,
        }
    }

    pub fn accept_match_event(&mut self, match_event: PartialMatchEvent<'p>) {
        let callback = |instance: &mut MatchInstance<'p>| {
            self.runner.run(instance, &match_event);
        };

        let window_bound = match_event.start_time.saturating_sub(self.window_size);

        self.storage.query(&match_event, window_bound, callback);

        self.runner.store_new_instances(&mut self.storage);
    }
}

impl<'p, P> Iterator for CompositionLayer<'p, P>
where
    P: Iterator<Item = PartialMatchEvent<'p>>,
{
    type Item = (u32, MatchInstance<'p>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.runner.output_buffer.is_empty() {
            let match_event = self.prev_layer.next()?;
            self.accept_match_event(match_event);
        }

        if let Some(output) = self.runner.output_buffer.last() {
            debug!("output: ({:?})", output);
        }
        self.runner.output_buffer.pop()
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::rc::Rc;

    use super::*;
    use crate::input_event::InputEvent;
    use crate::pattern::{Pattern, PatternEventType};
    use crate::process_layers::MatchingLayer;
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
    fn event(eid: u64, sub_id: u64, obj_id: u64, sig: &str) -> Vec<Rc<InputEvent>> {
        let sigs: Vec<&str> = sig.split('#').collect();
        vec![Rc::new(InputEvent::new(
            eid, eid, sigs[0], sub_id, sigs[1], obj_id, sigs[2],
        ))]
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
                .map(|x| x.event_ids[0])
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
        let match_layer =
            MatchingLayer::new(input.into_iter(), &pattern, &decomposition, window_size).unwrap();
        let mut layer = CompositionLayer::new(match_layer, &decomposition, window_size);

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
        let match_layer =
            MatchingLayer::new(input.into_iter(), &pattern, &decomposition, window_size).unwrap();
        let mut layer = CompositionLayer::new(match_layer, &decomposition, window_size);

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
        let match_layer =
            MatchingLayer::new(input.into_iter(), &pattern, &decomposition, window_size).unwrap();
        let mut layer = CompositionLayer::new(match_layer, &decomposition, window_size);

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
        let match_layer =
            MatchingLayer::new(input.into_iter(), &pattern, &decomposition, window_size).unwrap();
        let mut layer = CompositionLayer::new(match_layer, &decomposition, window_size);

        assert!(layer.next().is_none());
    }

    fn verify_event(
        match_event: &UniversalMatchEvent,
        time_pair: (u64, u64),
        entity_id_pair: (u64, u64),
        event_ids: &[u64],
    ) {
        assert_eq!(match_event.start_time, time_pair.0);
        assert_eq!(match_event.end_time, time_pair.1);
        assert_eq!(match_event.subject_id, entity_id_pair.0);
        assert_eq!(match_event.object_id, entity_id_pair.1);
        assert_eq!(*match_event.event_ids, *event_ids)
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
        let match_layer =
            MatchingLayer::new(input.into_iter(), &pattern, &decomposition, window_size).unwrap();
        let mut layer = CompositionLayer::new(match_layer, &decomposition, window_size);

        let match_events = layer.next().unwrap().1.match_events;
        verify_event(&match_events[0], (0, 0), (0, 1), &[0]);
        verify_event(&match_events[1], (1, 3), (1, 2), &[1, 2, 3]);
        verify_event(&match_events[2], (4, 4), (2, 3), &[4]);
        assert!(layer.next().is_none());
    }
}
