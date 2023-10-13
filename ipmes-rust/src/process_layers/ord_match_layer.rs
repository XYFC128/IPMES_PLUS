use crate::input_edge::InputEdge;
use crate::match_edge::MatchEdge;
use crate::pattern::Edge as PatternEdge;
use crate::sub_pattern::SubPattern;
use crate::sub_pattern_match::SubPatternMatch;
use itertools::Itertools;
use regex::Error as RegexError;
use regex::Regex;
use std::cmp::max;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
struct PartialMatch<'p> {
    timestamp: u64,
    node_id_map: Vec<u64>,
    edges: Vec<MatchEdge<'p>>,
}

impl<'p> PartialMatch<'p> {
    pub fn to_sub_pattern_match(self) -> SubPatternMatch {
        todo!()
    }
}

struct SubMatcher<'p> {
    signature: Regex,
    pattern_edge: &'p PatternEdge,
    sub_pattern_id: i64,
    buffer: VecDeque<PartialMatch<'p>>,
}

impl<'p> SubMatcher<'p> {
    pub fn new(pattern_edge: &'p PatternEdge, use_regex: bool) -> Result<Self, RegexError> {
        let match_syntax = if use_regex {
            format!("^{}$", pattern_edge.signature)
        } else {
            format!("^{}$", regex::escape(&pattern_edge.signature))
        };
        let signature = Regex::new(&match_syntax)?;

        Ok(Self {
            signature,
            pattern_edge,
            sub_pattern_id: -1,
            buffer: VecDeque::new(),
        })
    }
    pub fn match_against(&mut self, input_edges: &[Rc<InputEdge>]) -> Vec<PartialMatch<'p>> {
        input_edges
            .iter()
            .filter(|edge| self.signature.is_match(&edge.signature))
            .cartesian_product(self.buffer.iter())
            .filter_map(|(input_edge, partial_match)| {
                self.merge(Rc::clone(input_edge), partial_match)
            })
            .collect()
    }

    fn merge(
        &self,
        input_edge: Rc<InputEdge>,
        partial_match: &PartialMatch<'p>,
    ) -> Option<PartialMatch<'p>> {
        // Check node collision
        let start_id = partial_match.node_id_map[self.pattern_edge.start];
        let end_id = partial_match.node_id_map[self.pattern_edge.end];
        if start_id > 0 && start_id != input_edge.start {
            return None;
        }
        if end_id > 0 && end_id != input_edge.end {
            return None;
        }

        // Check edge uniqueness
        if partial_match
            .edges
            .iter()
            .find(|edge| edge.input_edge.id == input_edge.id)
            .is_some()
        {
            return None;
        }

        let mut node_id_map = partial_match.node_id_map.clone();
        let mut edges = partial_match.edges.clone();
        node_id_map[self.pattern_edge.start] = input_edge.start;
        node_id_map[self.pattern_edge.end] = input_edge.end;

        let match_edge = MatchEdge {
            input_edge,
            matched: self.pattern_edge,
        };
        edges.push(match_edge);

        Some(PartialMatch {
            timestamp: partial_match.timestamp,
            node_id_map,
            edges,
        })
    }

    pub fn clear_expired(&mut self, time_bound: u64) {
        while let Some(head) = self.buffer.front() {
            if head.timestamp < time_bound {
                self.buffer.pop_front();
            } else {
                break;
            }
        }
    }
}

pub struct OrdMatchLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEdge>>>,
{
    prev_layer: P,
    use_regex: bool,
    window_size: u64,
    sub_matchers: Vec<SubMatcher<'p>>,
}

impl<'p, P> OrdMatchLayer<'p, P>
where
    P: Iterator<Item = Vec<Rc<InputEdge>>>,
{
    pub fn new(
        prev_layer: P,
        decomposition: &'p [SubPattern],
        use_regex: bool,
        window_size: u64,
    ) -> Result<Self, RegexError> {
        let mut sub_matchers = Vec::new();
        for sub_pattern in decomposition {
            for edge in &sub_pattern.edges {
                sub_matchers.push(SubMatcher::new(edge, use_regex)?);
            }
            // get the first sub-matcher for this sub-pattern
            if let Some(first) = sub_matchers
                .iter_mut()
                .nth_back(sub_pattern.edges.len() - 1)
            {
                let max_node_id = sub_pattern
                    .edges
                    .iter()
                    .map(|e| max(e.start, e.end))
                    .max()
                    .unwrap();
                first.buffer.push_back(PartialMatch {
                    timestamp: u64::MAX,
                    node_id_map: vec![0; max_node_id + 1],
                    edges: vec![],
                })
            }
            if let Some(last) = sub_matchers.last_mut() {
                last.sub_pattern_id = sub_pattern.id as i64;
            }
        }

        Ok(Self {
            prev_layer,
            use_regex,
            window_size,
            sub_matchers,
        })
    }
}

impl<P> Iterator for OrdMatchLayer<'_, P>
where
    P: Iterator<Item = Vec<Rc<InputEdge>>>,
{
    type Item = Vec<SubPatternMatch>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut results = Vec::new();

        while results.is_empty() {
            let time_batch = self.prev_layer.next()?;
            let time_bound = if let Some(edge) = time_batch.first() {
                edge.timestamp.saturating_sub(self.window_size)
            } else {
                continue;
            };

            let mut prev_result = Vec::new();
            for matcher in &mut self.sub_matchers {
                matcher.clear_expired(time_bound);

                matcher.buffer.extend(prev_result.into_iter());
                let cur_result = matcher.match_against(&time_batch);
                if matcher.sub_pattern_id != -1 {
                    results.extend(cur_result.into_iter().map(|m| m.to_sub_pattern_match()));
                    prev_result = Vec::new();
                } else {
                    prev_result = cur_result;
                }
            }
        }

        Some(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_input_edge(id: u64, signature: &str) -> Rc<InputEdge> {
        Rc::new(InputEdge {
            timestamp: 0,
            signature: signature.to_string(),
            id,
            start: 1,
            end: 2,
        })
    }

    fn input_edge(id: u64, signature: &str, start: u64, end: u64) -> Rc<InputEdge> {
        Rc::new(InputEdge {
            timestamp: 0,
            signature: signature.to_string(),
            id,
            start,
            end,
        })
    }
    #[test]
    fn test_sub_matcher_no_regex() {
        let pattern_edge = PatternEdge {
            id: 0,
            signature: "edge*".to_string(),
            start: 0,
            end: 1,
        };

        let sub_pattern = SubPattern {
            id: 0,
            edges: vec![&pattern_edge],
        };

        let decomposition = &[sub_pattern];

        let mut matcher = SubMatcher::new(&pattern_edge, false).unwrap();
        matcher.buffer.push_back(PartialMatch {
            timestamp: u64::MAX,
            node_id_map: vec![0; 2],
            edges: vec![],
        });

        let input_edges = vec![
            simple_input_edge(1, "edge*"),
            simple_input_edge(2, "edgee"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "input_edge"),
        ];

        let result = matcher.match_against(&input_edges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edges[0].input_edge.id, 1);
    }

    #[test]
    fn test_sub_matcher_regex() {
        let pattern_edge = PatternEdge {
            id: 0,
            signature: "edge*".to_string(),
            start: 0,
            end: 1,
        };

        let sub_pattern = SubPattern {
            id: 0,
            edges: vec![&pattern_edge],
        };

        let decomposition = &[sub_pattern];

        let mut matcher = SubMatcher::new(&pattern_edge, true).unwrap();
        matcher.buffer.push_back(PartialMatch {
            timestamp: u64::MAX,
            node_id_map: vec![0; 2],
            edges: vec![],
        });

        let input_edges = vec![
            simple_input_edge(1, "edge*"),
            simple_input_edge(2, "edgee"),
            simple_input_edge(3, "edge1"),
            simple_input_edge(4, "input_edge"),
        ];

        let result = matcher.match_against(&input_edges);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edges[0].input_edge.id, 2);
    }

    #[test]
    fn test_ord_match_layer() {
        let pattern_edge1 = PatternEdge {
            id: 0,
            signature: "edge1".to_string(),
            start: 0,
            end: 1,
        };
        let pattern_edge2 = PatternEdge {
            id: 1,
            signature: "edge2".to_string(),
            start: 1,
            end: 2,
        };

        let sub_pattern = SubPattern {
            id: 0,
            edges: vec![&pattern_edge1, &pattern_edge2],
        };
        let decomposition = [sub_pattern];

        let time_batch = vec![
            input_edge(2, "edge2", 2, 3),
            input_edge(3, "foo", 4, 5),
            input_edge(4, "bar", 6, 7),
            input_edge(1, "edge1", 1, 2),
        ];

        let mut layer =
            OrdMatchLayer::new([time_batch].into_iter(), &decomposition, false, u64::MAX).unwrap();
        let result = layer.next().unwrap();
        assert_eq!(result.len(), 1);
    }
}
