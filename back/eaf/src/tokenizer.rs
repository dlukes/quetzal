//! Detect token boundaries in transcribed segment.
//!
//! Tokenization is relatively dumb, it just divides the input text into fairly
//! simple token categories (cf. `TokenKind`). In particular, it doesn't
//! attempt to detect any mistakes, not even whether non-whitespace tokens
//! consist of allowed sequences of characters. This is all done as part of
//! parsing, so that all mistakes are collected at one point, and also so that
//! tokenization errors don't prevent further processing, because ideally, we
//! want to inform about as many errors as possible at the same time.
//!
//! Whitespace is normalized prior to tokenization, as this isn't something
//! we'd want people to fix by hand.

use lazy_static::lazy_static;
use regex::{Match, Regex, RegexBuilder};

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
}

pub fn tokenize(source: &str) -> Tokenized {
    lazy_static! {
        static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
        static ref TOKENIZER_RE: Regex = RegexBuilder::new(
            r#"
            # paired delimiter token:
                [
                    \[\]\(\)<>
                ]
            |
            # whitespace:
                \s+
            |
            # non-whitespace:
                [^
                    \[\]\(\)<>
                    \s
                ]+
        "#
        )
        .ignore_whitespace(true)
        .build()
        .unwrap();
    }
    // normalize whitespace
    let source = WHITESPACE_RE.replace_all(source.trim(), " ").into_owned();
    let tokens = TOKENIZER_RE
        .find_iter(&source)
        .filter_map::<Token, _>(|m| {
            if m.as_str() == " " {
                None
            } else {
                Some(Token::from(m))
            }
        })
        .collect();
    Tokenized { source, tokens }
}

#[cfg(test)]
mod tests {
    use super::{DelimKind::*, TokenKind::*, *};

    #[test]
    fn tokenize_square_brackets() {
        let seg = tokenize("foo [bar] baz");
        assert_eq!(seg.tokens[1].kind, Open(Square));
        assert_eq!(seg.tokens[3].kind, Close(Square));
    }

    #[test]
    fn tokenize_round_brackets() {
        let seg = tokenize("foo (bar) baz");
        assert_eq!(seg.tokens[1].kind, Open(Round));
        assert_eq!(seg.tokens[3].kind, Close(Round));
    }

    #[test]
    fn tokenize_angle_brackets() {
        let seg = tokenize("foo <bar> baz");
        assert_eq!(seg.tokens[1].kind, Open(Angle));
        assert_eq!(seg.tokens[3].kind, Close(Angle));
    }

    fn compare_tokens(source: &str, tokens: &[&str]) {
        let segment = tokenize(source);
        assert_eq!(
            segment.tokens.len(),
            tokens.len(),
            "Number of tokens differs."
        );
        for (tokenized, reference) in segment.tokens.iter().zip(tokens.iter()) {
            let tokenized = segment.as_str(tokenized);
            eprintln!("tokenized = {:?} :: reference = {:?}", tokenized, reference);
            assert_eq!(&tokenized, reference, "Token values as str differ.");
        }
    }

    #[test]
    fn compare_nice() {
        compare_tokens(
            "čáp [dřepí @ <SM v] .. (louži>)",
            &[
                "čáp", "[", "dřepí", "@", "<", "SM", "v", "]", "..", "(", "louži", ">", ")",
            ],
        );
    }

    #[test]
    fn compare_not_nice() {
        compare_tokens(
            "foo][ bar(baz)..",
            &["foo", "]", "[", "bar", "(", "baz", ")", ".."],
        );
    }
}
