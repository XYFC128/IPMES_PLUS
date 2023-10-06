use crate::input::InputEdge;

struct ParseLayer {

}

impl Iterator for ParseLayer {
    type Item = Vec<InputEdge>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}