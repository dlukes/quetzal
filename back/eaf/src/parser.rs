//! Check whether sequence of tokens in segment is structurally valid.
//!
//! This is where *all* kinds of mistakes are detected and recorded. If there
//! are any, the user will thus get a full list of what's wrong, so that they
//! can fix everything in one go.
use lazy_static::lazy_static;
use regex::{Matches, Regex};

use crate::{DelimKind::*, Mistake, Node, Parsed, Token, TokenKind::*, Tokenized};
use std::cmp::Reverse;

#[derive(Debug)]
pub struct ParserConfig {
    /// Full tokens that are explicitly allowed.
    whitelist: Option<Regex>,
    /// Full tokens that are explicitly disallowed.
    blacklist: Option<Regex>,
    /// Graphemes and grapheme sequences hich are allowed in tokens not covered by the above.
    atoms: Option<Regex>,
    /// Codes allowed in a _-separated list after <.
    after_angle: Option<Regex>,
}

impl ParserConfig {
    pub fn from_args<W, B, A, G>(
        whitelist: &[W],
        blacklist: &[B],
        atoms: &[A],
        after_angle: &[G],
    ) -> Self
    where
        W: std::borrow::Borrow<str>,
        B: std::borrow::Borrow<str>,
        A: std::borrow::Borrow<str> + Clone,
        G: std::borrow::Borrow<str>,
    {
        let mut atoms = atoms.to_vec();
        atoms.sort_unstable_by_key(|x| Reverse(x.borrow().len()));
        let joined = atoms.join("|");
        let atoms = if joined.is_empty() {
            None
        } else {
            Some(Regex::new(&joined).unwrap())
        };

        Self {
            whitelist: Self::slice_to_regex(whitelist),
            blacklist: Self::slice_to_regex(blacklist),
            atoms,
            after_angle: Self::slice_to_regex(after_angle),
        }
    }

    fn slice_to_regex<S: std::borrow::Borrow<str>>(slice: &[S]) -> Option<Regex> {
        let joined = slice.join("|");
        if joined.is_empty() {
            None
        } else {
            Some(Regex::new(&format!(r"\A(?:{})\z", joined)).unwrap())
        }
    }
}

impl ParserConfig {
    fn is_match(opt_re: &Option<Regex>, s: &str) -> bool {
        opt_re.as_ref().map(|re| re.is_match(s)).unwrap_or_default()
    }

    fn in_whitelist(&self, s: &str) -> bool {
        Self::is_match(&self.whitelist, s)
    }

    fn in_blacklist(&self, s: &str) -> bool {
        Self::is_match(&self.blacklist, s)
    }

    fn in_after_angle(&self, s: &str) -> bool {
        Self::is_match(&self.after_angle, s)
    }

    fn maybe_iter_atoms<'r, 't>(&'r self, s: &'t str) -> Option<Matches<'r, 't>> {
        self.atoms.as_ref().map(|re| re.find_iter(s))
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

        lazy_static! {
            static ref NUMERIC_RE: Regex = Regex::new(r"-?\d*?[,\.]?\d+").unwrap();
        }

        if NUMERIC_RE.is_match(token_str) {
            // plain numbers should only be allowed inside parens as counts
            // of unintelligible words
            if self.round_start.is_none() {
                word_ok = false;
                self.mistakes.push(Mistake::BadToken { at: self.current });
            }
        } else if self.config.in_whitelist(token_str) {
        } else if self.config.in_blacklist(token_str) {
            word_ok = false;
            self.mistakes.push(Mistake::BadToken { at: self.current });
        } else if let Some(atoms) = self.config.maybe_iter_atoms(token_str) {
            let token_len = token_str.len();
            let mut prev_end = 0;
            for atom in atoms {
                let (start, end) = (atom.start(), atom.end());
                if start != prev_end {
                    word_ok = false;
                    self.mistakes.push(Mistake::BadSubstr {
                        start: prev_end,
                        end: start,
                        at: self.current,
                    })
                }
                prev_end = end;
            }
            if prev_end != token_len {
                self.mistakes.push(Mistake::BadSubstr {
                    start: 0,
                    end: token_len,
                    at: self.current,
                })
            }
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
            if self.config.in_after_angle(&code) {
                if !(code.is_empty() || codes.contains(&code)) {
                    codes.push(code);
                }
            } else {
                codes_ok = false;
                self.mistakes.push(Mistake::BadAttr {
                    attr: code,
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
    use super::*;
    use crate::tokenizer;

    lazy_static! {
        static ref ATOMS: Vec<String> = {
            let mut atoms = ('A'..='Z')
                .chain('a'..='z')
                .map(|c| c.to_string())
                .collect::<Vec<_>>();
            atoms.push("č".to_string());
            atoms.push("á".to_string());
            atoms.push("d͡ʒ".to_string());
            atoms
        };
        static ref CONFIG: ParserConfig =
            ParserConfig::from_args(&[r"\.", r"\.\.", "@", "#li", "&"], &["hm"], &ATOMS, &["SM"]);
    }

    #[test]
    fn test_config() {
        // NOTE: only tests after_angle, but the other ones should work exactly
        // the same (the regexes are prepared and matched the same way)

        let pc = ParserConfig::from_args::<&str, &str, &str, _>(&[], &[], &[], &["SM", "SJ"]);
        assert!(pc.in_after_angle("SM"));
        assert!(pc.in_after_angle("SJ"));
        assert!(
            !pc.in_after_angle("SM_SJ"),
            "the regex is meant to match one code at a time"
        );
        assert!(
            !pc.in_after_angle("SMSJ"),
            "the regex is meant to match one code at a time"
        );
        assert!(!pc.in_after_angle("MJ"));
        assert!(!pc.in_after_angle(""));
        assert!(!pc.in_after_angle("_"));
        assert!(!pc.in_after_angle("_SM"));

        let pc = ParserConfig::from_args::<&str, &str, &str, _>(&[], &[], &[], &["SM"]);
        assert!(pc.in_after_angle("SM"));
        assert!(!pc.in_after_angle(""));
        assert!(!pc.in_after_angle("_"));
        assert!(!pc.in_after_angle("_SM"));
        assert!(!pc.in_after_angle("SJ"));
        assert!(!pc.in_after_angle("SM_SJ"));

        let pc = ParserConfig::from_args::<&str, &str, &str, &str>(&[], &[], &[], &[]);
        assert!(!pc.in_after_angle("SM"));
        assert!(
            !pc.in_after_angle(""),
            "the empty string should never be valid"
        );
        assert!(!pc.in_after_angle("_"));

        let pc = ParserConfig::from_args::<&str, &str, &str, _>(&[], &[], &[], &[""]);
        assert!(!pc.in_after_angle("SM"));
        assert!(
            !pc.in_after_angle(""),
            "the empty string should never be valid"
        );
        assert!(!pc.in_after_angle("_"));
    }

    #[test]
    fn test_whitelist() {
        assert!(!ATOMS.iter().any(|s| s == "."));
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize(".."));
        assert!(!seg.has_mistakes());
        assert_eq!(
            seg.nodes[0],
            Node::Token(Token {
                kind: NonDelim,
                start: 0,
                end: 2
            })
        );
    }

    #[test]
    fn test_blacklist() {
        assert!(ATOMS.iter().any(|s| s == "h"));
        assert!(ATOMS.iter().any(|s| s == "m"));
        assert!(&CONFIG.in_blacklist("hm"));
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("hm"));
        assert!(seg.has_mistakes());
        assert_eq!(seg.mistakes[0], Mistake::BadToken { at: 0 });
    }

    #[test]
    fn test_disallowed_atoms() {
        assert!(!ATOMS.iter().any(|s| s == "ž"));
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("ž"));
        assert!(seg.has_mistakes());
        assert_eq!(
            seg.mistakes[0],
            Mistake::BadSubstr {
                start: 0,
                end: 2,
                at: 0
            }
        );
    }

    #[test]
    fn test_multi_codepoint_atoms() {
        let seg = Parser::parse(&CONFIG, tokenizer::tokenize("d͡ʒi d͡zi ʒi"));
        assert!(seg.has_mistakes());
        assert_eq!(seg.mistakes.len(), 2);
        assert_eq!(
            seg.mistakes[0],
            Mistake::BadSubstr {
                start: 1,
                end: 3,
                at: 1,
            }
        );
        assert_eq!(
            seg.mistakes[1],
            Mistake::BadSubstr {
                start: 0,
                end: 2,
                at: 2,
            }
        );
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
        assert_eq!(
            seg.mistakes[0],
            Mistake::BadSubstr {
                start: 1,
                end: 2,
                at: 1
            }
        );
    }

    macro_rules! test_delims {
        ($fname:ident, $kind:path, $source:expr) => {
            #[test]
            fn $fname() {
                let seg = Parser::parse(&CONFIG, tokenizer::tokenize($source));
                assert_eq!(seg.mistakes.len(), 3);
                assert_eq!(
                    seg.mistakes[0],
                    Mistake::ClosingUnopenedDelim { kind: $kind, at: 0 }
                );
                assert_eq!(
                    seg.mistakes[1],
                    Mistake::NestedDelim {
                        kind: $kind,
                        outermost_start: 1,
                        at: 2
                    }
                );
                assert_eq!(
                    seg.mistakes[2],
                    Mistake::UnclosedDelim { kind: $kind, at: 1 }
                );
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
