use std::{ops::Deref, rc::Rc};

use super::InputEvent;
use std::hash::Hash;

/// A [Rc<InputEvent>] wrapper for the use in the case that we only care about the id of the [InputEvent].
/// The hashing and comparison of [IdSensitiveInputEvent] will only be performed on its id
#[derive(Clone)]
pub struct IdSensitiveInputEvent(Rc<InputEvent>);

impl Hash for IdSensitiveInputEvent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.event_id.hash(state)
    }
}

impl PartialEq for IdSensitiveInputEvent {
    fn eq(&self, other: &Self) -> bool {
        self.0.event_id == other.0.event_id
    }
}

impl Eq for IdSensitiveInputEvent {}

impl Deref for IdSensitiveInputEvent {
    type Target = InputEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Rc<InputEvent>> for IdSensitiveInputEvent {
    fn from(value: Rc<InputEvent>) -> Self {
        IdSensitiveInputEvent(value)
    }
}

impl Into<Rc<InputEvent>> for IdSensitiveInputEvent {
    fn into(self) -> Rc<InputEvent> {
        self.0
    }
}
