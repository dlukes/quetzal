use std::iter::repeat;

use unicode_segmentation::UnicodeSegmentation;

mod parser;
mod tokenizer;

#[derive(Debug)]
enum TokenKind {
    Word,
    Special,
    ParaCode,
    Count,
    OpenRound,
    CloseRound,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
    OpenCurly,
    CloseCurly,
    UnexpectedGrapheme,
}

#[derive(Debug)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

impl Token {
    fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token { kind, start, end }
    }
}

#[derive(Debug)]
struct Segment {
    tokens: Vec<Token>,
    text: String,
}

impl Segment {
    fn new(text: &str) -> Self {
        Segment {
            tokens: vec![],
            text: text.to_owned(),
        }
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token::new(kind, start, end));
    }

    fn debug(&self) {
        for tok in &self.tokens {
            let space_len = self.text[..tok.start].graphemes(true).count();
            let caret_len = self.text[tok.start..tok.end].graphemes(true).count();
            let highlight: String = repeat(' ')
                .take(space_len)
                .chain(repeat('^').take(caret_len))
                .collect();
            eprintln!("Token: {:?}", tok);
            eprintln!("{}", self.text);
            eprintln!("{}", highlight);
        }
    }
}
