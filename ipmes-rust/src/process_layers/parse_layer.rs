mod ordered_event;

use crate::input_event::InputEvent;
use ::std::rc::Rc;
use csv::StringRecord;
use ordered_event::OrderedEvent;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;

fn parse_timestamp(input: &str) -> Option<u64> {
    let mut result = 0u64;
    let mut chars = input.chars();
    for c in chars.by_ref() {
        if c == '.' {
            break;
        }
        result = result * 10 + c.to_digit(10)? as u64;
    }
    for _ in 0..3 {
        let digit = chars.next().map_or(Some(0), |c| c.to_digit(10))?;
        result = result * 10 + digit as u64;
    }

    Some(result)
}

pub struct ParseLayer {
    reader: csv::Reader<File>,
    record: StringRecord,
    // a min heap
    buffer: BinaryHeap<OrderedEvent>,
    boundary_time: u64,
    event_count: u32,
}

enum RecordParseResult {
    Single(InputEvent),
    Dual(InputEvent, InputEvent),
}

impl ParseLayer {
    pub fn new(reader: csv::Reader<File>) -> Self {
        Self {
            reader,
            record: StringRecord::new(),
            buffer: BinaryHeap::new(),
            boundary_time: 0,
            event_count: 0,
        }
    }

    fn read_next_record(&mut self) -> bool {
        while !self.reader.is_done() {
            if self.reader.read_record(&mut self.record).is_ok() {
                return true;
            }
        }

        false
    }

    fn parse_record(&self) -> Option<RecordParseResult> {
        let timestamp1 = parse_timestamp(self.record.get(0)?)?;
        let timestamp2 = parse_timestamp(self.record.get(1)?)?;
        let event_id = self.record.get(2)?.parse::<u64>().ok()?;
        let event_sig = self.record.get(3)?;
        let subject_id = self.record.get(4)?.parse::<u64>().ok()?;
        let subject_sig = self.record.get(5)?;
        let object_id = self.record.get(6)?.parse::<u64>().ok()?;
        let object_sig = self.record.get(7)?;

        if timestamp1 != timestamp2 {
            Some(RecordParseResult::Dual(
                InputEvent::new(
                    timestamp1,
                    event_id,
                    event_sig,
                    subject_id,
                    subject_sig,
                    object_id,
                    object_sig,
                ),
                InputEvent::new(
                    timestamp2,
                    event_id,
                    event_sig,
                    subject_id,
                    subject_sig,
                    object_id,
                    object_sig,
                ),
            ))
        } else {
            Some(RecordParseResult::Single(InputEvent::new(
                timestamp1,
                event_id,
                event_sig,
                subject_id,
                subject_sig,
                object_id,
                object_sig,
            )))
        }
    }

    fn nothing_to_send(&self) -> bool {
        match self.buffer.peek() {
            Some(edge) => edge >= &self.boundary_time,
            None => true,
        }
    }

    fn get_batch(&mut self) -> Option<Box<[Rc<InputEvent>]>> {
        let mut edges_to_flush: Vec<Rc<InputEvent>> = Vec::new();
        loop {
            match self.buffer.peek() {
                Some(edge) if edge < &self.boundary_time => {
                    edges_to_flush.push(Rc::new(self.buffer.pop().unwrap().into()));
                }
                _ => {
                    break;
                }
            }
        }

        if !edges_to_flush.is_empty() {
            Some(edges_to_flush.into_boxed_slice())
        } else {
            None
        }
    }
}

impl Iterator for ParseLayer {
    type Item = Box<[Rc<InputEvent>]>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.nothing_to_send() {
            if self.read_next_record() {
                match self.parse_record() {
                    Some(RecordParseResult::Single(input)) => {
                        self.boundary_time = input.timestamp;
                        self.buffer.push(OrderedEvent::new(input, self.event_count));
                        self.event_count += 1;
                    }
                    Some(RecordParseResult::Dual(input1, input2)) => {
                        self.boundary_time = input1.timestamp;
                        self.buffer
                            .push(OrderedEvent::new(input1, self.event_count));
                        self.buffer
                            .push(OrderedEvent::new(input2, self.event_count + 1));
                        self.event_count += 2;
                    }
                    None => {}
                }
            } else {
                self.boundary_time = u64::MAX;
                break;
            }
        }
        self.get_batch()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_parsing() {
        assert_eq!(parse_timestamp("0"), Some(0));
        assert_eq!(parse_timestamp("100"), Some(100 * 1000));
        assert_eq!(parse_timestamp("1.123"), Some(1123));
        assert_eq!(parse_timestamp("1.12"), Some(1120));
        assert_eq!(parse_timestamp("1.1234"), Some(1123));
        assert_eq!(parse_timestamp("1."), Some(1000));
    }

    #[test]
    fn test() {
        let reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path("testcases/test.csv")
            .unwrap();
        let parse_layer = ParseLayer::new(reader);

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
