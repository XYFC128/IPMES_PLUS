use crate::match_event::MatchEvent;
use crate::process_layers::composition_layer;
use crate::process_layers::composition_layer::match_instance::{
    InputEntityId, InputEventId, PatternEntityId,
};
use crate::process_layers::join_layer::SubPatternBuffer;
use crate::universal_match_event::UniversalMatchEvent;
use log::debug;
use std::cmp::Ordering;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::rc::Rc;

/// Matches of sub-patterns.
#[derive(Clone)]
// pub struct SubPatternMatch<'p> {
pub struct SubPatternMatch {
    /// The timestamp of the last event (in `match_events`), which is also the latest timestamp; indicating "current time".
    pub latest_time: u64,
    /// The timestamp of the earliest event; for determining expiry of this match.
    pub earliest_time: u64,

    /// Sorted input event ids for event uniqueness determination.
    pub event_ids: Box<[InputEventId]>,

    /// `event_id_map[matched_id] = input_event`
    ///
    /// `event_id_map.len()` == number of event in the "whole pattern".
    ///
    /// > Note: The terms **matched event** and pattern event are used interchangeably.
    pub match_event_map: Box<[Option<Rc<MatchEvent>>]>,

    /// The id of the matched sub-pattern.
    pub id: usize,
    
    /// Sorted array of `(input entity id, pattern entity id)`.
    ///
    /// `match_entities.len()` == number of entities in this sub-pattern match.
    pub match_entities: Box<[(InputEntityId, PatternEntityId)]>,
}

// pub struct DebugMatchEventMap<'p, 't>(pub &'t [Option<UniversalMatchEvent<'p>>]);
pub struct DebugMatchEventMap<'t>(pub &'t [Option<MatchEvent>]);
impl<'p, 't> Debug for DebugMatchEventMap<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_list()
        //     .entries(
        //         self.0
        //             .iter()
        //             .map(|opt| opt.as_ref().map(|val| &val.raw_events.get_ids())),
        //     )
        //     .finish()
        !todo!("Fix debug display");
        std::fmt::Result::Ok(())
    }
}

// impl<'p> Debug for SubPatternMatch<'p> {
impl<'p> Debug for SubPatternMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("SubPatternMatch")
        //     .field("id", &self.id)
        //     .field("match_entities", &self.match_entities)
        //     .field("event_ids", &self.event_ids)
        //     .field(
        //         "match_event_map",
        //         &DebugMatchEventMap(&self.match_event_map),
        //     )
        //     .finish()

        !todo!("Fix debug display");
        std::fmt::Result::Ok(())
    }
}

/// > Note: Since pattern-edges in sub-patterns are disjoint, we need not check uniqueness.
fn merge_match_event_map<T>(
    event_map1: &[Option<Rc<T>>],
    event_map2: &[Option<Rc<T>>],
) -> Box<[Option<Rc<T>>]>
where
    T: Clone,
{
    assert_eq!(event_map1.len(), event_map2.len());
    let mut merged = Vec::with_capacity(event_map1.len());
    for (a, b) in event_map1.iter().zip(event_map2.iter()) {
        if a.is_some() {
            merged.push(a.clone());
        } else if b.is_some() {
            merged.push(b.clone());
        } else {
            merged.push(None);
        }
    }
    merged.into_boxed_slice()
}

fn try_merge_event_ids(id_list1: &[u64], id_list2: &[u64]) -> bool {
    let mut p1 = id_list1.iter();
    let mut p2 = id_list2.iter();
    let mut next1 = p1.next();
    let mut next2 = p2.next();

    while let (Some(id1), Some(id2)) = (next1, next2) {
        match id1.cmp(id2) {
            Ordering::Less => {
                next1 = p1.next();
            }
            Ordering::Equal => {
                debug!("event id duplicates: {}", id1);
                return false;
            }
            Ordering::Greater => {
                next2 = p2.next();
            }
        }
    }
    true
}

fn merge_event_ids(id_list1: &[u64], id_list2: &[u64]) -> Option<Box<[u64]>> {
    if !try_merge_event_ids(id_list1, id_list2) {
        return None;
    }

    let mut merged = Vec::with_capacity(id_list1.len() + id_list2.len());
    let mut p1 = id_list1.iter();
    let mut p2 = id_list2.iter();
    let mut next1 = p1.next();
    let mut next2 = p2.next();

    while let (Some(id1), Some(id2)) = (next1, next2) {
        match id1.cmp(id2) {
            Ordering::Less => {
                merged.push(*id1);
                next1 = p1.next();
            }
            Ordering::Equal => {
                debug!("event id duplicates: {}", id1);
                return None;
            }
            Ordering::Greater => {
                merged.push(*id2);
                next2 = p2.next();
            }
        }
    }

    if next1.is_none() {
        p1 = p2;
        next1 = next2;
    }

    while let Some(id) = next1 {
        merged.push(*id);
        next1 = p1.next();
    }

    Some(merged.into_boxed_slice())
}

pub fn try_merge_entities(a: &[(u64, u64)], b: &[(u64, u64)], max_num_entities: usize) -> bool {
    let mut used_entities = vec![false; max_num_entities];

    let mut p1 = a.iter();
    let mut p2 = b.iter();

    let mut next1 = p1.next();
    let mut next2 = p2.next();

    while let (Some(node1), Some(node2)) = (next1, next2) {
        if used_entities[node1.1 as usize] || used_entities[node2.1 as usize] {
            debug!("different input nodes match the same pattern");
            return false;
        }

        if node1.0 < node2.0 {
            used_entities[node1.1 as usize] = true;
            next1 = p1.next();
        } else if node1.0 > node2.0 {
            used_entities[node2.1 as usize] = true;
            next2 = p2.next();
        } else {
            if node1.1 != node2.1 {
                debug!("an input node matches distinct patterns");
                return false;
            }
            used_entities[node1.1 as usize] = true;
            next1 = p1.next();
            next2 = p2.next();
        }
    }

    if next1.is_none() {
        p1 = p2;
        next1 = next2;
    }

    while let Some(node) = next1 {
        if used_entities[node.1 as usize] {
            return false;
        }
        used_entities[node.1 as usize] = true;
        next1 = p1.next();
    }

    true
}

pub fn merge_entities(
    a: &[(u64, u64)],
    b: &[(u64, u64)],
    max_num_entities: usize,
) -> Option<Box<[(u64, u64)]>> {
    if !try_merge_entities(a, b, max_num_entities) {
        return None;
    }

    let mut merged = Vec::with_capacity(a.len() + b.len());

    let mut p1 = a.iter();
    let mut p2 = b.iter();

    let mut next1 = p1.next();
    let mut next2 = p2.next();

    while let (Some(node1), Some(node2)) = (next1, next2) {
        if node1.0 < node2.0 {
            merged.push(*node1);
            next1 = p1.next();
        } else if node1.0 > node2.0 {
            merged.push(*node2);
            next2 = p2.next();
        } else {
            merged.push(*node1);
            next1 = p1.next();
            next2 = p2.next();
        }
    }

    if next1.is_none() {
        p1 = p2;
        next1 = next2;
    }

    while let Some(node) = next1 {
        merged.push(*node);
        next1 = p1.next();
    }

    Some(merged.into_boxed_slice())
}

impl<'p> SubPatternMatch {
    pub fn build(
        sub_pattern_id: u32,
        match_instance: composition_layer::MatchInstance,
        num_pattern_event: usize,
    ) -> Option<Self> {
        let latest_time = match_instance
            .match_events
            .last()?
            .raw_events
            .get_interval()
            .1;
        let earliest_time = match_instance.start_time;

        // let match_events = match_instance.match_events.into_vec();
        let match_events = match_instance.match_events.into_vec();

        let match_entities = match_instance.match_entities.clone();

        let mut event_ids: Vec<u64> = match_events
            .iter()
            .flat_map(|e| e.raw_events.get_ids())
            .collect();
        event_ids.sort_unstable();

        let mut match_event_map = vec![None; num_pattern_event];
        for event in match_events.into_iter() {
            let pat_id = event.match_id as usize;
            match_event_map[pat_id] = Some(Rc::new(event));
        }

        Some(Self {
            id: sub_pattern_id as usize,
            latest_time,
            earliest_time,
            event_ids: event_ids.into_boxed_slice(),
            match_entities,
            match_event_map: match_event_map.into_boxed_slice(),
        })
    }

    pub fn merge_matches(
        sub_pattern_buffer: &SubPatternBuffer,
        sub_pattern_match1: &Self,
        sub_pattern_match2: &Self,
    ) -> Option<Self> {
        // debug!(
        //     "now try merging\n{:?} and\n{:?}",
        //     sub_pattern_match1, sub_pattern_match2,
        // );

        debug!("event uniqueness checking...");

        let event_ids =
            merge_event_ids(&sub_pattern_match1.event_ids, &sub_pattern_match2.event_ids)?;
        let match_event_map = merge_match_event_map(
            &sub_pattern_match1.match_event_map,
            &sub_pattern_match2.match_event_map,
        );

        debug!("order relation checking...");

        // check "order relation"
        if !sub_pattern_buffer
            .relation
            .check_order_relation(&match_event_map)
        {
            return None;
        }

        debug!("shared node and node uniqueness checking");

        // handle "shared node" and "node uniqueness"
        let match_entities = merge_entities(
            &sub_pattern_match1.match_entities,
            &sub_pattern_match2.match_entities,
            sub_pattern_buffer.max_num_entities,
        )?;

        Some(SubPatternMatch {
            // 'id' is meaningless here
            id: 0,
            latest_time: max(
                sub_pattern_match1.latest_time,
                sub_pattern_match2.latest_time,
            ),
            earliest_time: min(
                sub_pattern_match1.earliest_time,
                sub_pattern_match2.earliest_time,
            ),
            match_entities,
            event_ids,
            match_event_map,
        })
    }
}

/// Helper structure that implements `PartialEq`, `Ord`, `PartialOrd` traits for `SubPatternMatch`.
///
/// *Earliest* refers to `SubPatternMatch.earliest_time`.
#[derive(Clone, Debug)]
pub struct EarliestFirst(pub SubPatternMatch);

impl Eq for EarliestFirst {}

impl PartialEq<Self> for EarliestFirst {
    fn eq(&self, other: &Self) -> bool {
        self.0.earliest_time.eq(&other.0.earliest_time)
    }
}

impl Ord for EarliestFirst {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.earliest_time.cmp(&other.0.earliest_time).reverse()
    }
}

impl PartialOrd<Self> for EarliestFirst {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use crate::pattern::SubPattern;

    use super::*;

    #[test]
    fn test_merge_edge_id_map_1() {
        let edge_id_map1 = vec![None, Some(3), Some(2), None, None];
        let edge_id_map2 = vec![Some(1), None, None, None, Some(7)];

        // assert_eq!(
        //     [Some(1), Some(3), Some(2), None, Some(7)],
        //     *merge_match_event_map(&edge_id_map1, &edge_id_map2)
        // );
    }

    #[test]
    fn test_merge_edge_id_map_2() {
        let edge_id_map1 = vec![None, Some(3), None, None, None];
        let edge_id_map2 = vec![Some(1), None, None, None, Some(7)];

        // assert_eq!(
        //     [Some(1), Some(3), None, None, Some(7)],
        //     *merge_match_event_map(&edge_id_map1, &edge_id_map2)
        // );
    }

    #[test]
    fn test_merge_event_id_basecase() {
        let id_list1 = [1, 3, 5];
        let id_list2 = [2, 4];
        assert_eq!(
            *merge_event_ids(&id_list1, &id_list2).unwrap(),
            [1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn test_merge_event_id_dup_id() {
        let id_list1 = [1, 3, 5];
        let id_list2 = [3, 4];
        assert_eq!(merge_event_ids(&id_list1, &id_list2), None);
    }

    #[test]
    fn test_merge_event_id_edgecases() {
        assert_eq!(*merge_event_ids(&[1], &[2]).unwrap(), [1, 2]);
        assert_eq!(*merge_event_ids(&[2], &[1]).unwrap(), [1, 2]);
        assert_eq!(*merge_event_ids(&[1], &[]).unwrap(), [1]);
        assert!(merge_event_ids(&[], &[]).unwrap().is_empty(),);
    }

    #[test]
    /// shared node not shared between input nodes: Fail
    fn test_merge_entities1() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, &tmp_sub_pattern, max_node_id, 0);

        let a = vec![(2, 19), (7, 20), (11, 9)];

        let b = vec![(0, 17), (2, 22), (9, 11)];

        let ans = None;

        let merged = merge_entities(&a, &b, sub_pattern_buffer.max_num_entities);
        assert_eq!(merged, ans);
    }

    #[test]
    /// input node not unique: Fail
    fn test_merge_entities2() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, &tmp_sub_pattern, max_node_id, 0);

        let a = vec![(2, 19), (7, 20), (11, 9)];

        let b = vec![(0, 17), (25, 20)];
        let ans = None;

        let merged = merge_entities(&a, &b, sub_pattern_buffer.max_num_entities);
        assert_eq!(merged, ans);
    }

    #[test]
    /// Pass ("a" finished first)
    fn test_merge_entities3() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, &tmp_sub_pattern, max_node_id, 0);

        let a = vec![(2, 19), (7, 20), (11, 9)];

        let b = vec![(0, 17), (25, 27)];
        let ans = [(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];

        let merged = merge_entities(&a, &b, sub_pattern_buffer.max_num_entities);

        assert_ne!(merged, None);
        assert!(merged.unwrap().iter().eq(&ans));
    }

    #[test]
    /// Pass ("a" finished first)
    fn test_merge_entities4() {
        let max_node_id = 100;
        let tmp_sub_pattern = SubPattern {
            id: 0,
            events: vec![],
        };
        let sub_pattern_buffer = SubPatternBuffer::new(0, &tmp_sub_pattern, max_node_id, 0);

        let b = vec![(2, 19), (7, 20), (11, 9)];

        let a = vec![(0, 17), (25, 27)];
        let ans = [(0, 17), (2, 19), (7, 20), (11, 9), (25, 27)];

        let merged = merge_entities(&a, &b, sub_pattern_buffer.max_num_entities);

        assert_ne!(merged, None);
        assert!(merged.unwrap().iter().eq(&ans));
    }
}
