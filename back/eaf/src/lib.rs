use std::collections::HashSet;
use std::iter::repeat;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

lazy_static! {
    static ref WHITESPACE: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(Debug)]
enum TokenKind {
    Word,
    OpenRound,
    CloseRound,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
    OpenCurly,
    CloseCurly,
    AtSign,
    Ampersand,
    Dot,
    DoubleDot,
    Count,
    UnexpectedChar,
}

use TokenKind::*;

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
    fn with_text(text: &str) -> Self {
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

enum TokenizingState {
    Whitespace,
    InsideToken,
    OpenDelim,
    CloseDelim,
}

use TokenizingState::*;

struct Tokenizer {
    segment: Segment,
    state: TokenizingState,
    curr_tok_start: usize,
    word_chars: HashSet<char>,
    special_toks: HashSet<String>,
    special_tok_chars: HashSet<char>,
}

impl Tokenizer {
    fn new(text: &str, word_chars: &str, special_toks: &str) -> Self {
        let word_chars: HashSet<char> = word_chars.chars().collect();
        let special_toks: HashSet<String> = special_toks
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();
        let special_tok_chars: HashSet<char> =
            special_toks.iter().flat_map(|s| s.chars()).collect();
        Tokenizer {
            segment: Segment::with_text(text),
            state: Whitespace,
            curr_tok_start: 0,
            word_chars,
        }
    }

    fn tokenize(text: &str, word_chars: &str) -> Segment {
        // normalize whitespace for easier error reporting
        let text = WHITESPACE.replace_all(text, " ");
        let text = text.trim();
        let mut tokenizer = Tokenizer::new(text, word_chars);
        for ic in text.char_indices() {
            tokenizer.state = match tokenizer.state {
                Whitespace => tokenizer.leave_whitespace(ic),
                InsideToken => tokenizer.leave_inside_token(ic),
                OpenDelim => tokenizer.leave_open_delim(ic),
                CloseDelim => tokenizer.leave_close_delim(ic),
            }
        }
        tokenizer.segment
    }

    fn leave_whitespace(&mut self, (index, chr): (usize, char)) -> TokenizingState {
        match chr {
            '[' => {
                self.segment.push_token(OpenSquare, index, index + 1);
                OpenDelim
            }
            '@' => {
                self.segment.push_token(AtSign, index, index + 1);
                InsideToken
            }
            _ if chr.is_whitespace() => Whitespace,
            _ if self.word_chars.contains(&chr) => {
                self.curr_tok_start = index;
                InsideToken
            }
            _ => {
                self.segment
                    .push_token(UnexpectedChar, index, index + chr.len_utf8());
                InsideToken
            }
        }
    }

    fn leave_inside_token(&mut self, (index, chr): (usize, char)) -> TokenizingState {
        if self.word_chars.contains(&chr) {
            return InsideToken;
        }
        self.segment.push_token(Word, self.curr_tok_start, index);
        match chr {
            ']' => {
                self.segment.push_token(CloseSquare, index, index + 1);
                CloseDelim
            }
            _ if chr.is_whitespace() => Whitespace,
            _ => {
                self.segment
                    .push_token(UnexpectedChar, index, index + chr.len_utf8());
                InsideToken
            }
        }
    }

    fn leave_open_delim(&mut self, (index, chr): (usize, char)) -> TokenizingState {
        match chr {
            '@' => {
                self.segment.push_token(AtSign, index, index + 1);
                InsideToken
            }
            _ if self.word_chars.contains(&chr) => {
                self.curr_tok_start = index;
                InsideToken
            }
            _ => {
                self.segment
                    .push_token(UnexpectedChar, index, index + chr.len_utf8());
                InsideToken
            }
        }
    }

    fn leave_close_delim(&mut self, (index, chr): (usize, char)) -> TokenizingState {
        match chr {
            _ if chr.is_whitespace() => Whitespace,
            _ => {
                self.segment
                    .push_token(UnexpectedChar, index, index + chr.len_utf8());
                InsideToken
            }
        }
    }
}

// type StartIndex = usize;

// enum ParsingState {
//     Default,
//     InsideSquare(StartIndex),
// }

// use ParsingState::*;

// struct Parser {
//     state: ParsingState,
//     segment: Segment,
//     // index: usize,
// }

// impl Parser {
//     fn with_text(text: &str) -> Self {
//         Parser {
//             state: Default,
//             segment: Segment::with_text(text),
//             // index: 0,
//         }
//     }

//     fn parse(text: &str) -> Self {
//         let text = text.trim();
//         let mut parser = Self::with_text(text);
//         for ic in text.char_indices() {
//             parser.state = match parser.state {
//                 Default => parser.leave_default(ic),
//                 InsideSquare(_) => parser.leave_inside_square(ic),
//             }
//         }
//         unimplemented!()
//     }

//     fn leave_default(&self, (index, chr): (usize, char)) -> ParsingState {
//         match chr {
//             '[' => {
//                 self.segment.tokens.push(Token::new(OpenSquare, index, index + 1));
//                 InsideSquare(index)
//             },
//             ']' => ,
//             '@' => ,
//         }
//         unimplemented!()
//     }

//     fn leave_inside_square(&self, (index, chr): (usize, char)) -> ParsingState {
//         unimplemented!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer() {
        let seg = Tokenizer::tokenize("foo [čar @ baz] qux", "fočbarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [čar @ baz] qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [bar@ baz] qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[bar @ baz] qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [bar @ baz]qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[bar @ baz ]qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[ bar @ baz ]qux", "fobarzqux");
        seg.debug();
        eprintln!("=======================================================");
        assert!(false);
    }
}
