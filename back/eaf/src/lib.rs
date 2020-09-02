use std::iter::repeat;

use regex::Match;
use unicode_segmentation::UnicodeSegmentation;

pub mod parser;
pub mod tokenizer;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Whitespace,
    NonWhitespace,
    OpenRound,
    CloseRound,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
}

use TokenKind::*;

#[derive(Debug, Copy, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

impl<'t> From<Match<'t>> for Token {
    fn from(mat: Match) -> Self {
        let kind = match mat.as_str() {
            "(" => OpenRound,
            ")" => CloseRound,
            "[" => OpenSquare,
            "]" => CloseSquare,
            "<" => OpenAngle,
            ">" => CloseAngle,
            " " => Whitespace,
            _ => NonWhitespace,
        };
        Self {
            kind,
            start: mat.start(),
            end: mat.end(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DelimKind {
    Round,
    Square,
    Angle,
}

#[derive(Debug)]
pub enum Mistake {
    BadChar {
        token: Token,
        char: char,
        at: usize,
    },
    NestedDelim {
        kind: DelimKind,
        outermost_start: usize,
        at: usize,
    },
    ClosingUnopenedDelim {
        kind: DelimKind,
        at: usize,
    },
    UnclosedDelim {
        kind: DelimKind,
        at: usize,
    },
}

#[derive(Debug)]
pub struct Segment {
    pub source: String,
    pub tokens: Vec<Token>,
    pub mistakes: Vec<Mistake>,
}

impl Segment {
    pub fn as_str(&self, token: &Token) -> &str {
        &self.source[token.start..token.end]
    }

    pub fn has_mistakes(&self) -> bool {
        self.mistakes.len() > 0
    }

    pub fn highlight(&self, token: &Token) -> String {
        let space_len = self.source[..token.start].graphemes(true).count();
        let caret_len = self.source[token.start..token.end].graphemes(true).count();
        let highlight: String = repeat(' ')
            .take(space_len)
            .chain(repeat('^').take(caret_len))
            .collect();
        format!("{}\n{}", self.source, highlight)
    }

    pub fn debug(&self) {
        for tok in &self.tokens {
            let highlight = self.highlight(tok);
            eprintln!("Token: {:?}", tok);
            eprintln!("{}", self.source);
            eprintln!("{}", highlight);
        }
    }
}
