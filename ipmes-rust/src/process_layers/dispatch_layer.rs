use std::rc::Rc;

mod event_matcher;
mod normal_event_matcher;

use crate::{input_event::InputEvent, match_event::MatchEvent};
use crate::pattern::Event as PatternEvent;

type OrderInDecomposition = usize;

pub struct DispatchLayer<'p, P> {
    prev_layer: P,
    result: Vec<(OrderInDecomposition, MatchEvent<'p>)>,
}

impl<'p, P> Iterator for DispatchLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEvent>>>,
{
    type Item = (OrderInDecomposition, MatchEvent<'p>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.result.is_empty() {
            let time_batch = self.prev_layer.next()?;
            
        }

        self.result.pop()
    }
}