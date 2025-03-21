use std::fmt::Debug;

use crate::pattern::PatternEvent;
use crate::process_layers::matching_layer::PartialMatchEvent;

#[derive(Clone)]
pub struct UniversalMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub start_time: u64,
    pub end_time: u64,
    pub subject_id: u64,
    pub object_id: u64,
    pub event_ids: Box<[u64]>,
}

impl<'p> Debug for UniversalMatchEvent<'p> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UniversalMatchEvent")
            .field("matched", &self.matched.id)
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("subject_id", &self.subject_id)
            .field("object_id", &self.object_id)
            .field("event_ids", &self.event_ids)
            .finish()
    }
}

impl<'p> From<&PartialMatchEvent<'p>> for UniversalMatchEvent<'p> {
    fn from(value: &PartialMatchEvent<'p>) -> Self {
        let input_events = vec![value.input_event.event_id].into_boxed_slice();

        Self {
            matched: value.matched,
            start_time: value.start_time,
            end_time: value.input_event.timestamp,
            subject_id: value.subject_id,
            object_id: value.input_event.object_id,
            event_ids: input_events,
        }
    }
}
