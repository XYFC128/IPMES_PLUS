use crate::pattern::PatternEvent;

use super::filter::FilterInfo;

#[derive(Debug, Clone, Copy)]
pub enum SharedNodeInfo {
    None,
    Subject,
    Object,
    Both,
}

impl From<FilterInfo> for SharedNodeInfo {
    fn from(value: FilterInfo) -> Self {
        match value {
            FilterInfo::None => SharedNodeInfo::None,
            FilterInfo::MatchIdxOnly { match_idx: _ } => SharedNodeInfo::None,
            FilterInfo::Subject {
                match_idx: _,
                subject: _,
            } => SharedNodeInfo::Subject,
            FilterInfo::Object {
                match_idx: _,
                object: _,
            } => SharedNodeInfo::Object,
            FilterInfo::Endpoints {
                match_idx: _,
                subject: _,
                object: _,
            } => SharedNodeInfo::Both,
        }
    }
}

pub struct SinglePattern<'p> {
    pub pattern: &'p PatternEvent,
    pub match_idx: usize,
    pub shared_node_info: SharedNodeInfo,
    pub signature_idx: usize,
}

pub struct FreqPattern<'p> {
    pub pattern: &'p PatternEvent,
    pub match_idx: usize,
    pub shared_node_info: SharedNodeInfo,
    pub signature_idx: usize,
    pub frequency: u32,
}

pub struct FlowPattern<'p> {
    pub pattern: &'p PatternEvent,
    pub match_idx: usize,
    pub shared_node_info: SharedNodeInfo,
    pub src_sig_idx: usize,
    pub dst_sig_idx: usize,
}

pub enum PatternInfo<'p> {
    Single(SinglePattern<'p>),
    Freq(FreqPattern<'p>),
    Flow(FlowPattern<'p>),
}

impl<'p> From<SinglePattern<'p>> for PatternInfo<'p> {
    fn from(value: SinglePattern<'p>) -> Self {
        PatternInfo::Single(value)
    }
}

impl<'p> From<FreqPattern<'p>> for PatternInfo<'p> {
    fn from(value: FreqPattern<'p>) -> Self {
        PatternInfo::Freq(value)
    }
}

impl<'p> From<FlowPattern<'p>> for PatternInfo<'p> {
    fn from(value: FlowPattern<'p>) -> Self {
        PatternInfo::Flow(value)
    }
}
