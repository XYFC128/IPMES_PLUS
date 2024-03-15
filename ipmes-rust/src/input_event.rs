use std::cmp::Ordering;

/// Input event, which is an arc of the provenance graph.
#[derive(Eq, Debug, Clone)]
pub struct InputEvent {
    /// Records when this event occurs.
    pub timestamp: u64,
    /// The id of this event
    pub event_id: u64,
    /// The signature of this event.
    pub event_signature: String,
    /// The node (entity) where this arc is from.
    pub subject_id: u64,
    /// The signature of subject.
    pub subject_signature: String,
    /// The node (entity) where this arc goes to.
    pub object_id: u64,
    /// The signature of object.
    pub object_signature: String,
}

impl Ord for InputEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for InputEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl PartialEq<Self> for InputEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event_id == other.event_id
    }
}
