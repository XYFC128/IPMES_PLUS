use std::cmp::Ordering;

mod id_sensitive_input_event;
pub use id_sensitive_input_event::IdSensitiveInputEvent;

/// Input event, which is an arc of the provenance graph.
#[derive(Eq, Debug, Clone)]
pub struct InputEvent {
    /// Records when this event occurs.
    pub timestamp: u64,
    /// The id of this event
    pub event_id: u64,
    /// The node (entity) where this arc is from.
    pub subject_id: u64,
    /// The node (entity) where this arc goes to.
    pub object_id: u64,
    signatures: String,
    subject_sig_start: usize,
    object_sig_start: usize,
}

impl InputEvent {
    pub fn new(
        timestamp: u64,
        event_id: u64,
        event_signature: &str,
        subject_id: u64,
        subject_signature: &str,
        object_id: u64,
        object_signature: &str,
    ) -> Self {
        let mut signatures = String::with_capacity(
            event_signature.len() + subject_signature.len() + object_signature.len() + 2,
        );
        signatures.push_str(event_signature);
        signatures.push('\0');
        signatures.push_str(subject_signature);
        signatures.push('\0');
        signatures.push_str(object_signature);

        let subject_sig_start = event_signature.len() + 1;
        let object_sig_start = subject_sig_start + subject_signature.len() + 1;

        Self {
            timestamp,
            event_id,
            subject_id,
            object_id,
            signatures,
            subject_sig_start,
            object_sig_start,
        }
    }

    /// Returns the signature of this event, of the subject entity and of the object entity concatenated
    /// into a single string, seperated by the `'\0'` character.
    pub fn get_signatures(&self) -> &str {
        &self.signatures
    }

    pub fn get_event_signature(&self) -> &str {
        &self.signatures[..self.subject_sig_start - 1]
    }

    pub fn get_subject_signature(&self) -> &str {
        &self.signatures[self.subject_sig_start..self.object_sig_start - 1]
    }

    pub fn get_object_signature(&self) -> &str {
        &self.signatures[self.object_sig_start..]
    }
}

impl Ord for InputEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for InputEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

impl PartialEq<Self> for InputEvent {
    fn eq(&self, other: &Self) -> bool {
        self.event_id == other.event_id
    }
}
