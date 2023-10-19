use crate::pattern_match::PatternMatch;
use crate::sub_pattern_match::SubPatternMatch;

struct NaiveJoinLayer<P>
    where
        P: Iterator<Item = Vec<SubPatternMatch>>,
{
    prev_layer: P,
}

impl<P> Iterator for NaiveJoinLayer<P>
    where
        P: Iterator<Item = Vec<SubPatternMatch>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}