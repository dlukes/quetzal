use std::iter::repeat;

use unicode_segmentation::UnicodeSegmentation;

mod parser;
mod tokenizer;

use crate::tokenizer::Token;

#[derive(Debug)]
pub struct Segment {
    source: String,
    tokens: Vec<Token>,
}

impl Segment {
    fn as_str(&self, token: &Token) -> &str {
        &self.source[token.start..token.end]
    }

    fn highlight(&self, token: &Token) -> String {
        let space_len = self.source[..token.start].graphemes(true).count();
        let caret_len = self.source[token.start..token.end].graphemes(true).count();
        let highlight: String = repeat(' ')
            .take(space_len)
            .chain(repeat('^').take(caret_len))
            .collect();
        format!("{}\n{}", self.source, highlight)
    }

    fn debug(&self) {
        for tok in &self.tokens {
            let highlight = self.highlight(tok);
            eprintln!("Token: {:?}", tok);
            eprintln!("{}", self.source);
            eprintln!("{}", highlight);
        }
    }
}
