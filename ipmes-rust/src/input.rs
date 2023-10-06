use log::debug;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::error::Error;

#[derive(Eq, Debug)]
pub struct InputEdge {
    pub timestamp: u64,
    pub signature: String,
    pub id: u64,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    pub timestamp1: f64,
    pub timestamp2: f64,
    pub signature: String,
    pub id: u64,
    pub start: u64,
    pub end: u64,
}

pub fn parse(filename: &str) -> Result<(), Box<dyn Error>> {
    println!("hi");
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(filename)?;
    for data in csv.deserialize() {
        let record: Record = data?;
        let timestamp1: u64 = (record.timestamp1 * 1000.0).round() as u64;
        let timestamp2: u64 = (record.timestamp2 * 1000.0).round() as u64;

        let edge1 = InputEdge {
            timestamp: timestamp1,
            signature: record.signature.clone(),
            id: record.id,
            start: record.start,
            end: record.end,
        };

        let edge2 = InputEdge {
            timestamp: timestamp2,
            signature: record.signature,
            id: record.id,
            start: record.start,
            end: record.end,
        };

        // println!("edge1: {:#?}\n edge2:{:#?}\n\n", edge1, edge2);
    }
    Ok(())
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
        // println!("test");
        let status = parse("src/testcases/test.csv");
    }
}
