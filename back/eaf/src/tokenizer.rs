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
use regex::{Regex, RegexBuilder};

use crate::{Segment, Token, TokenKind};

pub fn tokenize(source: &str) -> Segment {
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
        .map::<Token, _>(From::from)
        .filter(|t| t.kind != TokenKind::Whitespace)
        .collect();
    Segment {
        source,
        tokens,
        mistakes: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TokenKind::*;

    #[test]
    fn tokenize_square_brackets() {
        let seg = tokenize("foo [bar] baz");
        assert_eq!(seg.tokens[1].kind, OpenSquare);
        assert_eq!(seg.tokens[3].kind, CloseSquare);
    }

    #[test]
    fn tokenize_round_brackets() {
        let seg = tokenize("foo (bar) baz");
        assert_eq!(seg.tokens[1].kind, OpenRound);
        assert_eq!(seg.tokens[3].kind, CloseRound);
    }

    #[test]
    fn tokenize_angle_brackets() {
        let seg = tokenize("foo <bar> baz");
        assert_eq!(seg.tokens[1].kind, OpenAngle);
        assert_eq!(seg.tokens[3].kind, CloseAngle);
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
