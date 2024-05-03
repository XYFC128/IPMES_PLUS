use crate::input_event::InputEvent;
use ::std::rc::Rc;
use csv::DeserializeRecordsIter;
use log::warn;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;

#[derive(Debug, serde::Deserialize)]
struct Record {
    pub timestamp1: f64,
    pub timestamp2: f64,
    pub id: u64,
    pub signature: String,
    pub subject_id: u64,
    pub subject_signature: String,
    pub object_id: u64,
    pub object_signature: String,
}

fn parse(record: Record) -> (InputEvent, Option<InputEvent>) {
    // let record: Record = data;
    let timestamp1: u64 = (record.timestamp1 * 1000.0).round() as u64;
    let timestamp2: u64 = (record.timestamp2 * 1000.0).round() as u64;

    let edge1 = InputEvent {
        timestamp: timestamp1,
        event_id: record.id,
        event_signature: record.signature,
        subject_id: record.subject_id,
        subject_signature: record.subject_signature,
        object_id: record.object_id,
        object_signature: record.object_signature,
    };

    if timestamp1 != timestamp2 {
        let edge2 = InputEvent {
            timestamp: timestamp2,
            ..edge1.clone()
        };
        (edge1, Some(edge2))
    } else {
        (edge1, None)
    }
}

pub struct ParseLayer<'a> {
    csv_iter: DeserializeRecordsIter<'a, File, Record>,
    // a min heap
    buffer: BinaryHeap<Reverse<InputEvent>>,
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
            match result {
                Ok(record) => return Some(record),
                Err(e) => warn!("Error occurred in input file: {e}"),
            };
        }
        None
    }

    fn nothing_to_send(&self) -> bool {
        match self.buffer.peek() {
            Some(edge) => edge.0.timestamp >= self.boundary_time,
            None => true,
        }
    }

    fn get_batch(&mut self) -> Option<Vec<Rc<InputEvent>>> {
        let mut edges_to_flush: Vec<Rc<InputEvent>> = Vec::new();
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
    type Item = Vec<Rc<InputEvent>>;

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
            .from_path("testcases/test.csv")
            .unwrap();
        let parse_layer = ParseLayer::new(&mut csv);

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
