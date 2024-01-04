use std::cmp::Ordering;

/// Input event, which is an arc of the provenance graph.
#[derive(Eq, Debug, Clone)]
pub struct InputEvent {
    /// Records when this event occurs.
    pub timestamp: u64,
    /// The labeling of this event.
    pub signature: String,
    pub id: u64,
    /// The node (entity) where this arc is from.
    pub subject: u64,
    /// The node (entity) where this arc goes to.
    pub object: u64,
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
        self.id == other.id
    }
}
