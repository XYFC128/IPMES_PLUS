use std::fmt::Debug;

/// Encodes the information about how to get the entity_id from a list of events.
/// 
/// For a given entity, the encoded information includes:
/// - The index of the event in the list.
/// - Whether the entity is the subject or the object of the event.
///
/// Suppose the target entity is the subject of the *i*-th event, the encoding works as follows:
/// - The information is stored in a 32 bits integer
/// - The high 31 bits `[31:1]` represent `i`
/// - The lowest bit: 0 indicates the subject, 1 indicates the object
#[derive(Clone, Copy, Eq, PartialEq)]
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
            encode: 1 + ((event_idx as u32) << 1),
        }
    }

    pub fn get_entity<E, F>(&self, events: &[E], endpoints_extractor: F) -> Option<u64>
    where
        F: Fn(&E) -> (u64, u64),
    {
        let index = self.get_index();
        if index < events.len() {
            Some(self.get_entity_unchecked(events, endpoints_extractor))
        } else {
            None
        }
    }

    pub fn get_entity_unchecked<E, F>(&self, events: &[E], endpoints_extractor: F) -> u64
    where
        F: Fn(&E) -> (u64, u64),
    {
        let index = self.get_index();
        let (subject, object) = endpoints_extractor(&events[index]);
        if self.is_object() {
            object
        } else {
            subject
        }
    }

    fn get_index(&self) -> usize {
        (self.encode >> 1) as usize
    }

    pub fn is_subject(&self) -> bool {
        self.encode & 1 == 0
    }

    pub fn is_object(&self) -> bool {
        self.encode & 1 == 1
    }
}

impl Debug for EntityEncode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let index = (self.encode >> 1) as usize;
        if self.is_object() {
            write!(f, "object_of({})", index)
        } else {
            write!(f, "subject_of({})", index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let enc = EntityEncode::subject_of(1);
        assert_eq!(enc.get_index(), 1);
        assert!(enc.is_subject());
        assert!(!enc.is_object());

        let enc = EntityEncode::object_of(1);
        assert_eq!(enc.get_index(), 1);
        assert!(!enc.is_subject());
        assert!(enc.is_object());
    }

    #[test]
    fn test_get_entity() {
        let entities = [(0, 1), (2, 3)];
        let extractor = |x: &(u64, u64)| *x;

        let enc = EntityEncode::subject_of(1);
        assert_eq!(enc.get_entity(&entities, extractor), Some(2));

        let enc = EntityEncode::object_of(2);
        assert_eq!(enc.get_entity(&entities, extractor), None);
    }
}
