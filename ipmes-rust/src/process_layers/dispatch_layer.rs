use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;

struct DispatchLayer<P>
where
    P: Iterator<Item = Vec<InputEdge>>,
{
    prev_layer: P,
}

impl<P> Iterator for DispatchLayer<P>
where
    P: Iterator<Item = Vec<InputEdge>>,
{
    type Item = Vec<MatchEdge>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}