use super::sub_pattern_match::EarliestFirst;
use crate::match_event::MatchEvent;
use crate::pattern::Pattern;
use crate::pattern::SubPattern;
use crate::universal_match_event::UniversalMatchEvent;
use log::debug;
use std::collections::{BinaryHeap, HashSet};
use std::rc::Rc;

// use super::{get_parent_id, get_sibling_id};

/// A structure that holds order relations between sibling buffers.
#[derive(Clone, Debug)]
pub struct Relation {
    /// `shared_entities.len() == num_node`
    ///
    /// If node `i` is shared, `shared_entities[i] == true`.
    ///
    /// `i`: pattern node id
    ///
    /// (The "overall structure" has guaranteed nodes be shared properly, when performing "SubPatternMatch::try_merge_nodes()".)
    shared_entities: Vec<bool>,

    /// `event_orders: (pattern_id1, pattern_id2)`
    ///
    /// `pattern_id1` (`pattern_id2`, respectively) is the id of some left  (right, respectively) buffer on `JoinLayer::sub_pattern_buffers`, where the two buffers are siblings.
    event_orders: Vec<(usize, usize)>,
}

impl Relation {
    pub fn new() -> Self {
        Self {
            shared_entities: Vec::new(),
            event_orders: Vec::new(),
        }
    }

    /// Check whether order relations are violated between two pattern matches.
    pub fn check_order_relation(&self, match_event_map: &[Option<Rc<MatchEvent>>]) -> bool {
        for (idx1, idx2) in &self.event_orders {
            if let (Some(event1), Some(event2)) = (&match_event_map[*idx1], &match_event_map[*idx2])
            {
                if !Self::satisfy_order(event1, event2) {
                    return false;
                }
            } else {
                debug!("failed to get events specified in the order relation");
                return false;
            }
        }

        true
    }

    fn satisfy_order(

        event1: &MatchEvent,
        event2: &MatchEvent,
    ) -> bool {
        // event1.end_time <= event2.start_time
        event1.raw_events.get_interval().1 <= event2.raw_events.get_interval().0
    }

    pub fn is_entity_shared(&self, id: usize) -> bool {
        self.shared_entities[id]
    }
}

impl Default for Relation {
    fn default() -> Self {
        Self::new()
    }
}

/// A Buffer that holds sub-pattern matches that correspond to a certain sub-pattern.
#[derive(Clone, Debug)]
// pub struct SubPatternBuffer<'p> {
// pub struct SubPatternBuffer<'p> {
pub struct SubPatternBuffer {
    /// Buffer id.
    pub id: usize,
    /// Ids of pattern entities (nodes) contained in this sub-pattern.
    pub (super) node_id_list: HashSet<usize>,
    /// Ids of pattern events (edges) contained in this sub-pattern.
    edge_id_list: HashSet<usize>,
    /// A buffer that holds sub-pattern matches.
    pub(crate) buffer: BinaryHeap<EarliestFirst>,
    // pub(crate) buffer: BinaryHeap<EarliestFirst<'p>>,
    /// A buffer that holds newly came sub-pattern matches.
    pub(crate) new_match_buffer: BinaryHeap<EarliestFirst>,
    // pub(crate) new_match_buffer: BinaryHeap<EarliestFirst<'p>>,
    /// The order relations between the sub-pattern this buffer corresponds to and the one its sibling buffer corresponds to.
    pub relation: Relation,
    /// Number of entities in the overall pattern.
    pub max_num_entities: usize,
    /// Number of events in the overall pattern.
    pub max_num_events: usize,
}

impl<'p> SubPatternBuffer {
    pub fn new(
        id: usize,
        sub_pattern: &SubPattern,
        max_num_entities: usize,
        max_num_events: usize,
    ) -> Self {
        let mut node_id_list = HashSet::new();
        let mut edge_id_list = HashSet::new();
        for &edge in &sub_pattern.events {
            node_id_list.insert(edge.subject.id);
            node_id_list.insert(edge.object.id);
            edge_id_list.insert(edge.id);
        }
        Self {
            id,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities,
            max_num_events,
        }
    }

    /// Precalculate order relations between sibling buffers.
    pub fn generate_relations(
        pattern: &Pattern,
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
    ) -> Relation {
        let mut shared_entities = vec![false; pattern.entities.len()];
        let mut event_orders = Vec::new();

        // identify shared nodes
        for i in 0..pattern.entities.len() {
            if sub_pattern_buffer1.node_id_list.contains(&i)
                && sub_pattern_buffer2.node_id_list.contains(&i)
            {
                shared_entities[i] = true;
            }
        }

        // generate order-relation (new)
        // If the dependency of (src, tgt) exists, add the dependency into the list of order relations.
        // Note that ``src'' always precedes ``tgt''.
        for (src, tgt) in pattern.order.get_dependencies() {
            if (sub_pattern_buffer1.edge_id_list.contains(&src)
                && sub_pattern_buffer2.edge_id_list.contains(&tgt))
                || (sub_pattern_buffer2.edge_id_list.contains(&src)
                    && sub_pattern_buffer1.edge_id_list.contains(&tgt))
            {
                event_orders.push((src, tgt));
            }
        }

        Relation {
            shared_entities,
            event_orders,
        }
    }

    /// Merge two sub-pattern buffers into a new one.
    pub fn merge_buffers(
        sub_pattern_buffer1: &SubPatternBuffer,
        sub_pattern_buffer2: &SubPatternBuffer,
        new_buffer_id: usize,
    ) -> Self {
        let mut node_id_list = sub_pattern_buffer1.node_id_list.clone();
        let mut edge_id_list = sub_pattern_buffer1.edge_id_list.clone();
        node_id_list.extend(&sub_pattern_buffer2.node_id_list);
        edge_id_list.extend(&sub_pattern_buffer2.edge_id_list);

        Self {
            id: new_buffer_id,
            node_id_list,
            edge_id_list,
            buffer: BinaryHeap::new(),
            new_match_buffer: BinaryHeap::new(),
            relation: Relation::new(),
            max_num_entities: sub_pattern_buffer1.max_num_entities,
            max_num_events: sub_pattern_buffer1.max_num_events,
        }
    }
}