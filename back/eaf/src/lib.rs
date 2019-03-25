enum Token {
    Word(String),
    LeftParen,
    RightParen,
    LessThan,
    GreaterThan,
    LeftBrace,
    RightBrace
}
struct Segment {
    tokens: Vec<Token>
}
fn parse_segment(input: &str) -> Segment {
    for c in input.char_indices() {

    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
