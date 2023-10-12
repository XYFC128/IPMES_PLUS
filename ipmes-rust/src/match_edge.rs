use std::rc::Rc;
use crate::input_edge::InputEdge;
use crate::pattern::Edge as PatternEdge;

#[derive(Clone)]
pub struct MatchEdge<'a> {
    pub input_edge: Rc<InputEdge>,
    pub matched: &'a PatternEdge,
}

impl<'a> MatchEdge<'a> {
}