use std::collections::VecDeque;
use crate::input_edge::InputEdge;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::SubPatternMatch;

struct PartialMatch;

pub struct OrdMatchLayer<P>
where
    P: Iterator<Item = Vec<InputEdge>>,
{
    prev_layer: P,
    use_regex: bool,
    window_size: u64,
    all_signatures: Vec<&'_ str>,
    sub_pattern_ids: Vec<i64>,
    buffers: Vec<VecDeque<PartialMatch>>
}

impl<P> OrdMatchLayer<P>
    where
        P: Iterator<Item = Vec<InputEdge>>,
{
    fn new(prev_layer: P, decomposition: &[SubPattern], use_regex: bool, window_size: u64) -> Self {
        let mut all_signatures = Vec::new();
        let mut sub_pattern_ids = Vec::new();
        let mut buffers = Vec::new();
        for sub_pattern in decomposition {
            for edge in sub_pattern.edges {
                all_signatures.push(edge.signature.as_str());
                sub_pattern_ids.push(sub_pattern.id as i64);
                buffers.push(VecDeque::new())
            }
        }
        sub_pattern_ids.push(-1);

        Self {
            prev_layer,
            use_regex,
            window_size,
            all_signatures,
            sub_pattern_ids,
            buffers,
        }
    }
}

impl<P> Iterator for OrdMatchLayer<P>
where
    P: Iterator<Item = Vec<InputEdge>>,
{
    type Item = Vec<SubPatternMatch>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}