use std::cmp::Ordering;

#[derive(Eq, Debug, Clone)]
pub struct InputEdge {
    pub timestamp: u64,
    pub signature: String,
    pub id: u64,
    pub start: u64,
    pub end: u64,
}

impl Ord for InputEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for InputEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl PartialEq<Self> for InputEdge {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
