use std::rc::Rc;

use crate::input_event::InputEvent;
use crate::pattern::PatternEvent;
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
use match_instance::MatchInstance;
use state::*;

#[derive(Clone)]
pub struct UniversalMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub start_time: u64,
    pub end_time: u64,
    pub subject_id: u64,
    pub object_id: u64,
    pub input_events: Box<[Rc<InputEvent>]>,
}

impl<'p> From<&PartialMatchEvent<'p>> for UniversalMatchEvent<'p> {
    fn from(value: &PartialMatchEvent<'p>) -> Self {
        let input_events = vec![Rc::clone(&value.input_event)].into_boxed_slice();

        Self {
            matched: value.matched,
            start_time: value.input_event.timestamp,
            end_time: value.input_event.timestamp,
            subject_id: value.input_event.subject_id,
            object_id: value.input_event.object_id,
            input_events,
        }
    }
}

pub struct CompoLayer<'p, P> {
    prev_layer: P,
    window_size: u64,
    runner: InstanceRunner<'p>,
    storage: InstanceStorage<'p>,
}

impl<'p, P> CompoLayer<'p, P> {
    pub fn new(prev_layer: P, window_size: u64, decomposition: &[SubPattern<'p>]) -> Self {
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

        let window_bound = match_event
            .input_event
            .timestamp
            .saturating_sub(self.window_size);

        self.storage.query(&match_event, window_bound, callback);

        self.runner.store_new_instances(&mut self.storage);
    }
}

impl<'p, P> Iterator for CompoLayer<'p, P>
where
    P: Iterator<Item = PartialMatchEvent<'p>>,
{
    type Item = (u32, Box<[UniversalMatchEvent<'p>]>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.runner.output_buffer.is_empty() {
            let match_event = self.prev_layer.next()?;
            self.accept_match_event(match_event);
        }

        self.runner.output_buffer.pop()
    }
}
