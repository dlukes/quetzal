//! Check whether sequence of tokens in segment is structurally valid.
//!
//! This is where *all* kinds of mistakes are detected and recorded. If there
//! are any, the user will thus get a full list of what's wrong, so that they
//! can fix everything in one go.
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

    fn get_current(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn get_kind(&self, index: usize) -> TokenKind {
        self.tokens
            .get(index)
            .map_or(TokenKind::Whitespace, |token| token.kind.clone())
    }

    fn step(&mut self) {
        let current = &self.tokens[self.current];
        match current.kind {
            TokenKind::Whitespace => (),
            TokenKind::NonWhitespace => self.parse_word(),
            TokenKind::OpenRound => self.parse_open_round(),
            TokenKind::CloseRound => self.parse_close_round(),
            TokenKind::OpenSquare => self.parse_open_square(),
            TokenKind::CloseSquare => self.parse_close_square(),
            TokenKind::OpenAngle => self.parse_open_angle(),
            TokenKind::CloseAngle => self.parse_close_angle(),
        }
    }

    fn parse_word(&mut self) {
        // TODO test if actually consists of word chars
    }
    fn parse_open_round(&mut self) {
        // TODO can contain regular word tokens (cf. above) OR one number
    }
    fn parse_close_round(&mut self) {}
    fn parse_open_square(&mut self) {}
    fn parse_close_square(&mut self) {}
    fn parse_open_angle(&mut self) {
        // TODO test if followed by allowed meta/anom two-letter code(s)
    }
    fn parse_close_angle(&mut self) {}
}

struct Segment {
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

impl Segment {
    fn has_mistakes(&self) -> bool {
        self.mistakes.len() > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
