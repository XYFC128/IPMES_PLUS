use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Eq)]
pub struct InputEdge {
    pub timestamp: u64,
    pub signature: String,
    pub id: u64,
    pub start: u64,
    pub end: u64,
}
pub fn parse(string: &str) -> InputEdge {
    todo!()
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

pub struct Sorter {
    buffer: BinaryHeap<InputEdge>,
}
impl Sorter {
    pub fn put(&mut self, edge: InputEdge) -> SortedIter {
        let timestamp = edge.timestamp;
        self.buffer.push(edge);
        SortedIter {
            buffer: &mut self.buffer,
            timestamp,
        }
    }
}

pub struct SortedIter<'a> {
    buffer: &'a mut BinaryHeap<InputEdge>,
    timestamp: u64,
}

impl<'a> Iterator for SortedIter<'a> {
    type Item = Vec<InputEdge>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {

    }
}