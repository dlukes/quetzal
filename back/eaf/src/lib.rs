use std::iter::repeat;

use regex::Match;
use unicode_segmentation::UnicodeSegmentation;

pub mod parser;
pub mod tokenizer;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DelimKind {
    Round,
    Square,
    Angle,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    NonDelim,
    Open(DelimKind),
    Close(DelimKind),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

impl<'t> From<Match<'t>> for Token {
    fn from(mat: Match) -> Self {
        use DelimKind::*;
        use TokenKind::*;

        let kind = match mat.as_str() {
            "(" => Open(Round),
            ")" => Close(Round),
            "[" => Open(Square),
            "]" => Close(Square),
            "<" => Open(Angle),
            ">" => Close(Angle),
            _ => NonDelim,
        };
        Self {
            kind,
            start: mat.start(),
            end: mat.end(),
        }
    }
}

#[derive(Debug)]
pub struct Tokenized {
    pub source: String,
    pub tokens: Vec<Token>,
}

impl Tokenized {
    pub fn as_str(&self, token: &Token) -> &str {
        &self.source[token.start..token.end]
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
// NOTE: The Node could also just be a single struct per token, with
// optional information as to which kinds of spans (possibly with which
// attributes) it's contained in. Better for searching, worse for
// serialization, which is our primary use case here.
#[derive(Debug, PartialEq)]
pub enum Node {
    AttrList(Vec<String>),
    Open(DelimKind),
    Close(DelimKind),
    Token(Token),
}

#[derive(Debug)]
pub enum Mistake {
    // at is for token offsets
    BadToken {
        at: usize,
    },
    BadGrapheme {
        start: usize,
        len: usize,
        at: usize,
    },
    BadAttr {
        attr: String,
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
    MissingAttrs {
        at: usize,
    },
}

#[derive(Debug)]
pub struct Parsed {
    pub source: String,
    pub tokens: Vec<Token>,
    pub nodes: Vec<Node>,
    pub mistakes: Vec<Mistake>,
}

impl Parsed {
    pub fn has_mistakes(&self) -> bool {
        !self.mistakes.is_empty()
    }
}
