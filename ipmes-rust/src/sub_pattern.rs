use crate::pattern::{Edge, Pattern};

pub struct SubPattern<'a> {
    pub id: usize,
    pub edges: Vec<&'a Edge>,
}

impl<'a> SubPattern<'a> {
}

pub fn decompose(pattern: &Pattern) -> Vec<SubPattern> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::sub_pattern::SubPattern;

    #[cfg(test)]
    fn test () {
    }
}