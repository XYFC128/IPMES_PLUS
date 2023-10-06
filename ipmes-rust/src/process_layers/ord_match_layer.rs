use crate::match_edge::MatchEdge;
use crate::sub_pattern_match::SubPatternMatch;

struct OrdMatchLayer<P>
    where
        P: Iterator<Item = Vec<MatchEdge>>,
{
    prev_layer: P,
}

impl<P> Iterator for OrdMatchLayer<P>
    where
        P: Iterator<Item = Vec<MatchEdge>>,
{
    type Item = Vec<SubPatternMatch>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}