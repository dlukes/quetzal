use crate::tokenizer::{self, Token, TokenKind};

struct Node;

struct Mistake;

struct Parser {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

struct Segment {
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

impl Parser {
    fn parse(segment: tokenizer::Segment) -> Segment {
        let mut parser = Self {
            source: segment.source,
            tokens: segment.tokens,
            current: 0,
            nodes: vec![],
            mistakes: vec![],
        };
        let num_tokens = parser.tokens.len();
        while parser.current < num_tokens {
            parser.step();
        }
        unimplemented!()
    }

    fn get_current(&self) -> Token {
        self.tokens[self.current]
    }

    fn step(&mut self) {
        let current = &self.tokens[self.current];
        match current.kind {
            TokenKind::Whitespace => (),
            TokenKind::NonWhitespace => self.parse_non_whitespace(),
            TokenKind::OpenRound => self.parse_open_round(),
            TokenKind::CloseRound => self.parse_close_round(),
            TokenKind::OpenSquare => self.parse_open_square(),
            TokenKind::CloseSquare => self.parse_close_square(),
            TokenKind::OpenAngle => self.parse_open_angle(),
            TokenKind::CloseAngle => self.parse_close_angle(),
        }
    }

    fn parse_non_whitespace(&mut self) {}
}
