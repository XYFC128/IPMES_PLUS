use std::error::Error;
pub struct InputEdge;
// pub fn parse(string: &str) -> InputEdge {
pub fn parse(string: &str) -> Result<(), Box<dyn Error>> {
    let mut csv = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("./testcases/test.csv")?;
    for data in csv.records() {
        let record = data?;
        panic!("{:?}", record);
    }

    println!("hi");
    Ok(())
    // todo!()
}

pub struct Sorter;

impl Sorter {
    pub fn put(&mut self, edge: InputEdge) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        parse("a");
    }
}