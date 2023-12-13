use std::cmp::Ordering;

#[derive(Eq, Debug, Clone)]
pub struct InputEvent {
    pub timestamp: u64,
    pub signature: String,
    pub id: u64,
    pub subject: u64,
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
