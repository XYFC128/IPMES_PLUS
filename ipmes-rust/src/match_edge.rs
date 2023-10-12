use std::rc::Rc;
use crate::input_edge::InputEdge;
use crate::pattern::Edge as PatternEdge;

#[derive(Clone)]
pub struct MatchEdge<'p> {
    pub input_edge: Rc<InputEdge>,
    pub matched: &'p PatternEdge,
}

impl<'p> MatchEdge<'p> {
}