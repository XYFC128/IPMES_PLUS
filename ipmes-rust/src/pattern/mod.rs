pub mod darpa;
pub mod order_relation;
pub mod spade;
pub mod parser;

use order_relation::OrderRelation;

#[derive(Debug)]
pub struct Event {
    pub id: usize,
    pub signature: String,
    // start node id
    pub subject: usize,
    // end node id
    pub object: usize,
}

pub struct Pattern {
    pub events: Vec<Event>,
    pub order: OrderRelation,
    pub num_entities: usize
}