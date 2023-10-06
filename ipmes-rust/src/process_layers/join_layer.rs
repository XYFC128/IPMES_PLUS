use crate::pattern_match::PatternMatch;
use crate::sub_pattern_match::SubPatternMatch;

struct JoinLayer<P>
    where
        P: Iterator<Item = Vec<SubPatternMatch>>,
{
    prev_layer: P,
}

impl<P> Iterator for JoinLayer<P>
    where
        P: Iterator<Item = Vec<SubPatternMatch>>,
{
    type Item = PatternMatch;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}