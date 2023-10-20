use std::cmp::Reverse;
use crate::input_edge::InputEdge;
use csv::DeserializeRecordsIter;
use std::collections::BinaryHeap;
use std::fs::File;
use::std::rc::Rc;

#[derive(Debug, serde::Deserialize)]
struct Record {
    pub timestamp1: f64,
    pub timestamp2: f64,
    pub signature: String,
    pub id: u64,
    pub start: u64,
    pub end: u64,
}

fn parse(record: Record) -> (InputEdge, Option<InputEdge>) {
    // let record: Record = data;
    let timestamp1: u64 = (record.timestamp1 * 1000.0).round() as u64;
    let timestamp2: u64 = (record.timestamp2 * 1000.0).round() as u64;

    let edge1 = InputEdge {
        timestamp: timestamp1,
        signature: record.signature.clone(),
        id: record.id,
        start: record.start,
        end: record.end,
    };

    if timestamp1 != timestamp2 {
        let edge2 = InputEdge {
            timestamp: timestamp2,
            signature: record.signature,
            id: record.id,
            start: record.start,
            end: record.end,
        };
        (edge1, Some(edge2))
    } else {
        (edge1, None)
    }
}

// #[derive(serde::Deserialize)]
pub struct ParseLayer<'a> {
    csv_iter: DeserializeRecordsIter<'a, File, Record>,
    buffer: BinaryHeap<Reverse<InputEdge>>,
    boundary_time: u64,
}
impl<'a> ParseLayer<'a> {
    pub fn new(csv: &'a mut csv::Reader<File>) -> Self {
        Self {
            csv_iter: csv.deserialize(),
            buffer: BinaryHeap::new(),
            boundary_time: 0,
        }
    }

    fn next_valid_record(&mut self) -> Option<Record> {
        while let Some(result) = self.csv_iter.next() {
            if let Ok(record) = result {
                return Some(record);
            }
        }
        None
    }

    fn nothing_to_send(&self) -> bool {
        match self.buffer.peek() {
            Some(edge) => {
                edge.0.timestamp >= self.boundary_time
            },
            None => true,
        }
    }

    fn get_batch(&mut self) -> Option<Vec<Rc<InputEdge>>> {
        let mut edges_to_flush: Vec<Rc<InputEdge>> = Vec::new();
        loop {
            match self.buffer.peek() {
                Some(edge) if edge.0.timestamp < self.boundary_time => {
                    edges_to_flush.push(Rc::new(self.buffer.pop().unwrap().0));
                }
                _ => {
                    break;
                }
            }
        }

        if !edges_to_flush.is_empty() {
            Some(edges_to_flush)
        } else {
            None
        }
    }
}

impl Iterator for ParseLayer<'_> {
    type Item = Vec<Rc<InputEdge>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.nothing_to_send() {
            match self.next_valid_record() {
                Some(record) => {
                    let (edge1, edge2) = parse(record);
                    self.boundary_time = edge1.timestamp;
                    self.buffer.push(Reverse(edge1));
                    if let Some(edge) = edge2 {
                        self.buffer.push(Reverse(edge));
                    }
                }
                None => {
                    self.boundary_time = u64::MAX;
                    break;
                }
            }
        }
        self.get_batch()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path("testcases/test.csv").unwrap();
        let parse_layer = ParseLayer::new(&mut csv);

        let mut outer = 0;
        for batch in parse_layer {
            let mut time: u64 = 0;
            for edge in batch {
                if time == 0 {
                    time = edge.timestamp;
                } else {
                    assert_eq!(edge.timestamp, time);
                }
            }
        }
    }
}
