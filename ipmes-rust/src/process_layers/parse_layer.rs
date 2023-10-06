use crate::input_edge::InputEdge;
use serde::Deserialize;
use std::error::Error;

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

struct ParseLayer {

}

impl Iterator for ParseLayer {
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