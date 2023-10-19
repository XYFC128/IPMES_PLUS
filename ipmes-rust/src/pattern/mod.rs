pub mod darpa;
pub mod order_relation;
pub mod spade;
pub mod parser;

use order_relation::OrderRelation;

use clap::ValueEnum;
use std::io::BufRead;

#[derive(Debug)]
pub struct Edge {
    pub id: usize,
    pub signature: String,
    // start node id
    pub start: usize,
    // end node id
    pub end: usize,
}

pub struct Pattern {
    pub edges: Vec<Edge>,
    pub order: OrderRelation,
    pub num_nodes: usize
}