//! Check whether sequence of tokens in segment is structurally valid.
//!
//! This is where *all* kinds of mistakes are detected and recorded. If there
//! are any, the user will thus get a full list of what's wrong, so that they
//! can fix everything in one go.
use regex::Regex;

use crate::{DelimKind, Mistake, Segment, Token, TokenKind};

pub struct ParserConfig {
    after_angle_whitelist: Regex,
}

impl ParserConfig {
    pub fn from_args<S>(after_angle_whitelist: &[S]) -> Self
    where
        S: std::borrow::Borrow<str>,
    {
        let aaw = after_angle_whitelist.join("|");
        ParserConfig {
            after_angle_whitelist: Regex::new(&format!(r#"\A(?:{})(?:_(?:{}))*\z"#, aaw, aaw))
                .expect("invalid after angle whitelist regex"),
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    source: String,
    tokens: Vec<Token>,
    current: usize,
    mistakes: Vec<Mistake>,

    round_start: Option<usize>,
    square_start: Option<usize>,
    angle_start: Option<usize>,
}

impl Parser {
    // pub fn parse(config: &ParserConfig, segment: Segment) -> Segment {
    pub fn parse(segment: Segment) -> Segment {
        let mut parser = Self {
            source: segment.source,
            tokens: segment.tokens,
            mistakes: segment.mistakes,
            current: 0,

            round_start: None,
            square_start: None,
            angle_start: None,
        };

        let num_tokens = parser.tokens.len();
        while parser.current < num_tokens {
            parser.step();
        }
        if let Some(at) = parser.round_start {
            parser.mistakes.push(Mistake::UnclosedDelim {
                kind: DelimKind::Round,
                at,
            });
        }
        if let Some(at) = parser.square_start {
            parser.mistakes.push(Mistake::UnclosedDelim {
                kind: DelimKind::Square,
                at,
            });
        }
        if let Some(at) = parser.angle_start {
            parser.mistakes.push(Mistake::UnclosedDelim {
                kind: DelimKind::Angle,
                at,
            });
        }

        Segment {
            source: parser.source,
            tokens: parser.tokens,
            mistakes: parser.mistakes,
        }
    }

    // TODO: remove?
    // fn get_current(&self) -> &Token {
    //     &self.tokens[self.current]
    // }

    // TODO: remove?
    // fn get_kind(&self, index: usize) -> TokenKind {
    //     self.tokens
    //         .get(index)
    //         .map_or(TokenKind::Whitespace, |token| token.kind)
    // }

    fn step(&mut self) {
        let current = &self.tokens[self.current];
        match current.kind {
            // whitespace is removed by tokenizer
            TokenKind::Whitespace => unreachable!(),
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
        let token = &self.tokens[self.current];
        let token_str = &self.source[token.start..token.end];
        for (i, c) in token_str.char_indices() {
            // TODO: this is just an approximate placeholder test
            if !c.is_alphanumeric() {
                self.mistakes.push(Mistake::BadChar {
                    token: *token,
                    char: c,
                    at: i,
                });
            }
            // TODO: plain numbers should only be allowed inside parens
        }
        self.current += 1;
    }

    fn parse_open_round(&mut self) {
        if let Some(i) = self.round_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: DelimKind::Round,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.round_start = Some(self.current);
        }
        self.current += 1;
    }

    fn parse_close_round(&mut self) {
        if self.round_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: DelimKind::Round,
                at: self.current,
            })
        }
        self.current += 1;
    }

    // TODO: the following methods are basically copy-pastes of the two
    // previous ones; any abstraction possible? at least a macro?

    fn parse_open_square(&mut self) {
        if let Some(i) = self.square_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: DelimKind::Square,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.square_start = Some(self.current);
        }
        self.current += 1;
    }

    fn parse_close_square(&mut self) {
        if self.square_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: DelimKind::Square,
                at: self.current,
            })
        }
        self.current += 1;
    }

    fn parse_open_angle(&mut self) {
        // TODO: test if followed by allowed meta/anom/JO 2/3-letter code(s)
        // TODO: possibly sort and deduplicate the codes?
        if let Some(i) = self.angle_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: DelimKind::Angle,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.angle_start = Some(self.current);
        }
        self.current += 1;
    }

    fn parse_close_angle(&mut self) {
        if self.angle_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: DelimKind::Angle,
                at: self.current,
            })
        }
        self.current += 1;
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::*;
    use crate::tokenizer;

    lazy_static! {
        static ref CONFIG: ParserConfig = ParserConfig::from_args(&["SM"]);
    }

    #[test]
    fn test_config() {
        let pc = ParserConfig::from_args(&["SM", "SJ"]);
        eprintln!("{}", pc.after_angle_whitelist.as_str());
        assert!(pc.after_angle_whitelist.is_match("SM"));
        assert!(pc.after_angle_whitelist.is_match("SJ"));
        assert!(pc.after_angle_whitelist.is_match("SM_SJ"));
        assert!(!pc.after_angle_whitelist.is_match("SMSJ"));
        assert!(!pc.after_angle_whitelist.is_match("MJ"));
        assert!(!pc.after_angle_whitelist.is_match(""));
        assert!(!pc.after_angle_whitelist.is_match("_"));
        assert!(!pc.after_angle_whitelist.is_match("_SM"));

        let pc = ParserConfig::from_args(&["SM"]);
        assert!(pc.after_angle_whitelist.is_match("SM"));
        assert!(!pc.after_angle_whitelist.is_match(""));
        assert!(!pc.after_angle_whitelist.is_match("_"));
        assert!(!pc.after_angle_whitelist.is_match("_SM"));
        assert!(!pc.after_angle_whitelist.is_match("SJ"));
        assert!(!pc.after_angle_whitelist.is_match("SM_SJ"));
    }

    #[test]
    fn test_all_fine() {
        let seg = Parser::parse(tokenizer::tokenize("čarala bonga máro"));
        assert!(!seg.has_mistakes());
    }

    #[test]
    fn test_all_fine_and_complicated() {
        let seg = Parser::parse(tokenizer::tokenize("[čarala <SM bonga] (máro>)"));
        assert!(!seg.has_mistakes());
    }

    #[test]
    fn test_bad_char_in_word() {
        let seg = Parser::parse(tokenizer::tokenize("čarala b%nga máro"));
        assert!(seg.has_mistakes());
        assert_eq!(seg.mistakes.len(), 1);
        let m = &seg.mistakes[0];
        if let Mistake::BadChar { token, char, at } = m {
            assert_eq!(seg.as_str(token), "b%nga");
            assert_eq!(*char, '%');
            assert_eq!(*at, 1);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }
    }

    macro_rules! test_delims {
        ($fname:ident, $kind:path, $source:expr) => {
            #[test]
            fn $fname() {
                let seg = Parser::parse(tokenizer::tokenize($source));
                dbg!(&seg);
                assert_eq!(seg.mistakes.len(), 3, "Segment should have 3 mistakes.");

                let m1 = &seg.mistakes[0];
                if let Mistake::ClosingUnopenedDelim { kind, at } = m1 {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*at, 0);
                } else {
                    panic!("unexpected mistake at #1: {:?}", m1);
                }

                let m2 = &seg.mistakes[1];
                if let Mistake::NestedDelim {
                    kind,
                    outermost_start,
                    at,
                } = m2
                {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*outermost_start, 1);
                    assert_eq!(*at, 2);
                } else {
                    panic!("unexpected mistake at #2: {:?}", m2);
                }

                let m3 = &seg.mistakes[2];
                if let Mistake::UnclosedDelim { kind, at } = m3 {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*at, 1);
                } else {
                    panic!("unexpected mistake at #3: {:?}", m3);
                }
            }
        };
    }

    test_delims!(test_round, DelimKind::Round, ")((");
    test_delims!(test_square, DelimKind::Square, "][[");
    test_delims!(test_angle, DelimKind::Angle, "><<");
}
