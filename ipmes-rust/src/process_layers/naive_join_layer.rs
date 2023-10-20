use crate::pattern_match::PatternMatch;
use crate::sub_pattern_match::SubPatternMatch;

struct NaiveJoinLayer<'p, P>
    where
        P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    prev_layer: P,
}

impl<'p, P> Iterator for NaiveJoinLayer<'p, P>
    where
        P: Iterator<Item = Vec<SubPatternMatch<'p>>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}