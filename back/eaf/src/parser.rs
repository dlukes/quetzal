//! Check whether sequence of tokens in segment is structurally valid.
//!
//! This is where *all* kinds of mistakes are detected and recorded. If there
//! are any, the user will thus get a full list of what's wrong, so that they
//! can fix everything in one go.
use regex::Regex;

use crate::{DelimKind::*, Mistake, Node, Parsed, Token, TokenKind::*, Tokenized};

#[derive(Debug)]
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
            after_angle_whitelist: Regex::new(&format!(r#"\A(?:{})\z"#, aaw))
                .expect("invalid after angle whitelist regex"),
        }
    }
}

#[derive(Debug)]
pub struct Parser<'c> {
    config: &'c ParserConfig,

    source: String,
    tokens: Vec<Token>,
    current: usize,
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,

    round_start: Option<usize>,
    square_start: Option<usize>,
    angle_start: Option<usize>,
}

impl<'c> Parser<'c> {
    pub fn parse(config: &'c ParserConfig, segment: Tokenized) -> Parsed {
        let mut parser = Self {
            config,

            source: segment.source,
            tokens: segment.tokens,
            current: 0,
            mistakes: vec![],
            nodes: vec![],

            round_start: None,
            square_start: None,
            angle_start: None,
        };

        let num_tokens = parser.tokens.len();
        while parser.current < num_tokens {
            parser.step();
        }
        if let Some(at) = parser.round_start {
            parser
                .mistakes
                .push(Mistake::UnclosedDelim { kind: Round, at });
        }
        if let Some(at) = parser.square_start {
            parser
                .mistakes
                .push(Mistake::UnclosedDelim { kind: Square, at });
        }
        if let Some(at) = parser.angle_start {
            parser
                .mistakes
                .push(Mistake::UnclosedDelim { kind: Angle, at });
        }

        Parsed {
            source: parser.source,
            tokens: parser.tokens,
            nodes: parser.nodes,
            mistakes: parser.mistakes,
        }
    }

    fn step(&mut self) {
        let current = &self.tokens[self.current];
        match current.kind {
            // whitespace is removed by tokenizer
            NonDelim => self.parse_word(),
            Open(Round) => self.parse_open_round(),
            Close(Round) => self.parse_close_round(),
            Open(Square) => self.parse_open_square(),
            Close(Square) => self.parse_close_square(),
            Open(Angle) => self.parse_open_angle(),
            Close(Angle) => self.parse_close_angle(),
        }
    }

    fn get_token<'s>(current: usize, tokens: &[Token], source: &'s str) -> (Token, &'s str) {
        let token = tokens[current];
        let token_str = &source[token.start..token.end];
        (token, token_str)
    }

    fn parse_word(&mut self) {
        let mut word_ok = true;
        let (token, token_str) = Parser::get_token(self.current, &self.tokens, &self.source);
        for (i, c) in token_str.char_indices() {
            // TODO: this is just an approximate placeholder test
            if !c.is_alphanumeric() {
                word_ok = false;
                self.mistakes.push(Mistake::BadChar {
                    char: c,
                    char_at: i,
                    at: self.current,
                });
            }
            // TODO: plain numbers should only be allowed inside parens
        }
        if word_ok {
            self.nodes.push(Node::Token(token));
        }
        self.current += 1;
    }

    fn parse_open_round(&mut self) {
        if let Some(i) = self.round_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: Round,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.round_start = Some(self.current);
            self.nodes.push(Node::Open(Round));
        }
        self.current += 1;
    }

    fn parse_close_round(&mut self) {
        if self.round_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: Round,
                at: self.current,
            })
        } else {
            self.nodes.push(Node::Close(Round));
        }
        self.current += 1;
    }

    // TODO: the following methods are basically copy-pastes of the two
    // previous ones; any abstraction possible? at least a macro?

    fn parse_open_square(&mut self) {
        if let Some(i) = self.square_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: Square,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.square_start = Some(self.current);
            self.nodes.push(Node::Open(Square));
        }
        self.current += 1;
    }

    fn parse_close_square(&mut self) {
        if self.square_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: Square,
                at: self.current,
            })
        } else {
            self.nodes.push(Node::Close(Square));
        }
        self.current += 1;
    }

    fn parse_open_angle(&mut self) {
        if let Some(i) = self.angle_start {
            self.mistakes.push(Mistake::NestedDelim {
                kind: Angle,
                outermost_start: i,
                at: self.current,
            });
        } else {
            self.angle_start = Some(self.current);
            self.nodes.push(Node::Open(Angle));
        }
        self.current += 1;

        if self.current == self.tokens.len() {
            self.mistakes
                .push(Mistake::MissingAttrs { at: self.current });
            return;
        }

        // TODO: can't merge with previous condition without some refactoring,
        // as get_token will panic with index out of bounds if self.current
        // is equal to self.tokens.len()
        let (token, token_str) = Parser::get_token(self.current, &self.tokens, &self.source);
        if token.kind != NonDelim {
            self.mistakes
                .push(Mistake::MissingAttrs { at: self.current });
            return;
        }

        let mut codes = vec![];
        let mut codes_ok = true;
        for code in token_str.split('_') {
            let code = code.to_owned();
            if self.config.after_angle_whitelist.is_match(&code) {
                if !(code.is_empty() || codes.contains(&code)) {
                    codes.push(code);
                }
            } else {
                codes_ok = false;
                self.mistakes.push(Mistake::BadSymbol {
                    symbol: code,
                    at: self.current,
                });
            }
        }
        if codes_ok {
            codes.sort();
            self.nodes.push(Node::AttrList(codes));
        }
        self.current += 1;
    }

    fn parse_close_angle(&mut self) {
        if self.angle_start.take().is_none() {
            self.mistakes.push(Mistake::ClosingUnopenedDelim {
                kind: Angle,
                at: self.current,
            })
        } else {
            self.nodes.push(Node::Close(Angle));
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
        eprintln!("regex: {}", pc.after_angle_whitelist.as_str());

        assert!(pc.after_angle_whitelist.is_match("SM"));
        assert!(pc.after_angle_whitelist.is_match("SJ"));
        assert!(
            !pc.after_angle_whitelist.is_match("SM_SJ"),
            "the regex is meant to match one code at a time"
        );
        assert!(
            !pc.after_angle_whitelist.is_match("SMSJ"),
            "the regex is meant to match one code at a time"
        );
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
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("čarala bonga máro"));
        assert!(!seg.has_mistakes());

        for (t, n) in seg.tokens.iter().zip(seg.nodes.iter()) {
            let nt = Node::Token(*t);
            assert_eq!(nt, *n);
        }
    }

    #[test]
    fn test_all_fine_and_complicated() {
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("[čarala <SM bonga] (máro>)"));
        assert!(!seg.has_mistakes());

        let nodes = vec![
            Node::Open(Square),
            Node::Token(Token {
                kind: NonDelim,
                start: 1,
                end: 8,
            }),
            Node::Open(Angle),
            Node::AttrList(vec!["SM".to_owned()]),
            Node::Token(Token {
                kind: NonDelim,
                start: 13,
                end: 18,
            }),
            Node::Close(Square),
            Node::Open(Round),
            Node::Token(Token {
                kind: NonDelim,
                start: 21,
                end: 26,
            }),
            Node::Close(Angle),
            Node::Close(Round),
        ];

        for (n1, n2) in seg.nodes.iter().zip(nodes.iter()) {
            assert_eq!(n1, n2);
        }
    }

    #[test]
    fn test_bad_char_in_word() {
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("čarala b%nga máro"));
        assert!(seg.has_mistakes());
        assert_eq!(seg.mistakes.len(), 1);
        let m = &seg.mistakes[0];
        if let Mistake::BadChar { char, char_at, at } = m {
            assert_eq!(*char, '%');
            assert_eq!(*char_at, 1);
            assert_eq!(*at, 1);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }
    }

    macro_rules! test_delims {
        ($fname:ident, $kind:path, $source:expr) => {
            #[test]
            fn $fname() {
                let seg = Parser::parse(&CONFIG, tokenizer::tokenize($source));
                assert_eq!(seg.mistakes.len(), 3, "Segment should have 3 mistakes.");

                let m = &seg.mistakes[0];
                if let Mistake::ClosingUnopenedDelim { kind, at } = m {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*at, 0);
                } else {
                    panic!("unexpected mistake at #1: {:?}", m);
                }

                let m = &seg.mistakes[1];
                if let Mistake::NestedDelim {
                    kind,
                    outermost_start,
                    at,
                } = m
                {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*outermost_start, 1);
                    assert_eq!(*at, 2);
                } else {
                    panic!("unexpected mistake at #2: {:?}", m);
                }

                let m = &seg.mistakes[2];
                if let Mistake::UnclosedDelim { kind, at } = m {
                    assert_eq!(*kind, $kind);
                    assert_eq!(*at, 1);
                } else {
                    panic!("unexpected mistake at #3: {:?}", m);
                }
            }
        };
    }

    test_delims!(test_round, Round, ")((");
    test_delims!(test_square, Square, "][[");

    #[test]
    fn test_angle() {
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("><<"));
        assert_eq!(seg.mistakes.len(), 5, "Segment should have 5 mistakes.");

        let m = &seg.mistakes[0];
        if let Mistake::ClosingUnopenedDelim { kind, at } = m {
            assert_eq!(*kind, Angle);
            assert_eq!(*at, 0);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }

        let m = &seg.mistakes[1];
        if let Mistake::MissingAttrs { at } = m {
            assert_eq!(*at, 2);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }

        let m = &seg.mistakes[2];
        if let Mistake::NestedDelim {
            kind,
            outermost_start,
            at,
        } = m
        {
            assert_eq!(*kind, Angle);
            assert_eq!(*outermost_start, 1);
            assert_eq!(*at, 2);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }

        let m = &seg.mistakes[3];
        if let Mistake::MissingAttrs { at } = m {
            assert_eq!(*at, 3);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }

        let m = &seg.mistakes[4];
        if let Mistake::UnclosedDelim { kind, at } = m {
            assert_eq!(*kind, Angle);
            assert_eq!(*at, 1);
        } else {
            panic!("unexpected mistake: {:?}", m);
        }
    }
}
