/// Encodes the information about how to get the entity_id from a list of events
///
/// The information includes:
/// - The index of the event in the list.
/// - Whether the entity is the subject or the object of the event.
///
/// Suppose the target is the subject of the *i*-th event, the encoding works as follows:
/// - The information is stored in a 32 bits integer
/// - The high 31 bits `[31:1]` represent `i`
/// - The lowest bit: 0 indicates the subject, 1 indicates the object
#[derive(Clone, Copy)]
pub struct EntityEncode {
    encode: u32,
}

impl EntityEncode {
    pub fn subject_of(event_idx: usize) -> EntityEncode {
        EntityEncode {
            encode: (event_idx as u32) << 1,
        }
    }

    pub fn object_of(event_idx: usize) -> EntityEncode {
        EntityEncode {
            encode: 1 + (event_idx as u32) << 1,
        }
    }

    pub fn get_entity<E, F>(&self, events: &[E], endpoints_extractor: F) -> Option<u64>
    where
        F: Fn(&E) -> (u64, u64),
    {
        let index = (self.encode >> 1) as usize;
        if index > events.len() {
            None
        } else {
            Some(self.get_entity_unchecked(events, endpoints_extractor))
        }
    }

    pub fn get_entity_unchecked<E, F>(&self, events: &[E], endpoints_extractor: F) -> u64
    where
        F: Fn(&E) -> (u64, u64),
    {
        let index = (self.encode >> 1) as usize;
        let (subject, object) = endpoints_extractor(&events[index]);
        if self.encode & 1 == 1 {
            object
        } else {
            subject
        }
    }
}
