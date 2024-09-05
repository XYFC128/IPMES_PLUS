use super::Pattern;

struct DecomposedPattern {
    pattern: Pattern,
    subpattern_heads: Vec<usize>,
}
