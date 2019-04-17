use std::iter::repeat;

use lazy_static::lazy_static;
use regex::{Match, Regex, RegexBuilder};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
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
    let tokens = TOKENIZER_RE.find_iter(&source).map(From::from).collect();
    Segment { source, tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_square_brackets() {
        let seg = tokenize("foo [bar] baz");
        assert_eq!(seg.tokens[2].kind, OpenSquare);
        assert_eq!(seg.tokens[4].kind, CloseSquare);
    }

    #[test]
    fn tokenize_round_brackets() {
        let seg = tokenize("foo (bar) baz");
        assert_eq!(seg.tokens[2].kind, OpenRound);
        assert_eq!(seg.tokens[4].kind, CloseRound);
    }

    #[test]
    fn tokenize_angle_brackets() {
        let seg = tokenize("foo <bar> baz");
        assert_eq!(seg.tokens[2].kind, OpenAngle);
        assert_eq!(seg.tokens[4].kind, CloseAngle);
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
            "čáp [dřepí @ v] .. louži",
            &[
                "čáp", " ", "[", "dřepí", " ", "@", " ", "v", "]", " ", "..", " ", "louži",
            ],
        );
    }

    #[test]
    fn compare_not_nice() {
        compare_tokens(
            "foo][ bar(baz)..",
            &["foo", "]", "[", " ", "bar", "(", "baz", ")", ".."],
        );
    }
}
