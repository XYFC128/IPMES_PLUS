use crate::{pattern::PatternEvent, process_layers::matching_layer::PartialMatchEvent};

#[derive(Clone)]
pub struct UniversalMatchEvent<'p> {
    pub matched: &'p PatternEvent,
    pub start_time: u64,
    pub end_time: u64,
    pub subject_id: u64,
    pub object_id: u64,
    pub event_ids: Box<[u64]>,
}

impl<'p> From<&PartialMatchEvent<'p>> for UniversalMatchEvent<'p> {
    fn from(value: &PartialMatchEvent<'p>) -> Self {
        let input_events = vec![value.input_event.event_id].into_boxed_slice();

        Self {
            matched: value.matched,
            start_time: value.input_event.timestamp,
            end_time: value.input_event.timestamp,
            subject_id: value.input_event.subject_id,
            object_id: value.input_event.object_id,
            event_ids: input_events,
        }
    }
}
