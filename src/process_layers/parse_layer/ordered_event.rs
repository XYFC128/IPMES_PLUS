use crate::input_event::InputEvent;

pub struct OrderedEvent {
    event: InputEvent,
    order: u32,
}

impl OrderedEvent {
    pub fn new(event: InputEvent, order: u32) -> Self {
        Self { event, order }
    }
}

impl From<OrderedEvent> for InputEvent {
    fn from(val: OrderedEvent) -> Self {
        val.event
    }
}

impl Eq for OrderedEvent {}

impl PartialEq for OrderedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event.timestamp == other.event.timestamp && self.order == other.order
    }
}

impl Ord for OrderedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.event.timestamp == other.event.timestamp {
            self.order.cmp(&other.order).reverse()
        } else {
            self.event.timestamp.cmp(&other.event.timestamp).reverse()
        }
    }
}

impl PartialOrd for OrderedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<u64> for OrderedEvent {
    fn eq(&self, other: &u64) -> bool {
        self.event.timestamp.eq(other)
    }
}

impl PartialOrd<u64> for OrderedEvent {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.event.timestamp.cmp(other))
    }
}
