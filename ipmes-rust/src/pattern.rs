mod darpa;
pub mod order_relation;
mod spade;
mod parser;

use order_relation::OrderRelation;

use clap::ValueEnum;
use std::io::BufRead;

pub struct Edge {
    id: usize,
    signature: String,
    start: usize,
    end: usize,
}

pub struct Pattern {
    edges: Vec<Edge>,
    order: OrderRelation,
}