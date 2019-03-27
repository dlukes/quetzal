use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

use crate::{Segment, TokenKind};
use TokenKind::*;

lazy_static! {
    static ref WHITESPACE: Regex = Regex::new(r"\s+").unwrap();
}

enum TokenizingState {
    Delim,
    InsideWord,
    InsideSpecial,
    OpenBracket,
    CloseBracket,
}

use TokenizingState::*;

struct Conf {
    word_graphemes: HashSet<String>,
    delim_graphemes: HashSet<String>,
    special_toks: HashSet<String>,
    special_graphemes: HashSet<String>,
    para_codes: HashSet<String>,
}

fn str2graphemes(string: &str) -> HashSet<String> {
    string.graphemes(true).map(|g| g.to_owned()).collect()
}

fn str2substrings(string: &str) -> HashSet<String> {
    string.split_whitespace().map(|s| s.to_owned()).collect()
}

impl Conf {
    fn new(
        word_graphemes: &str,
        delim_graphemes: &str,
        special_toks: &str,
        para_codes: &str,
    ) -> Self {
        let word_graphemes = str2graphemes(word_graphemes);
        let mut delim_graphemes = str2graphemes(delim_graphemes);
        delim_graphemes.insert(" ".to_owned());
        let special_graphemes: HashSet<String> = special_toks
            .graphemes(true)
            .filter(|g| !WHITESPACE.is_match(g))
            .map(|g| g.to_owned())
            .collect();
        let special_toks = str2substrings(special_toks);
        let para_codes = str2substrings(para_codes);
        // let special_toks: HashSet<String> = special_toks.into_iter().collect();
        // let para_codes
        // let special_tok_chars: HashSet<char> =
        //     special_toks.iter().flat_map(|s| s.chars()).collect();
        Conf {
            word_graphemes,
            delim_graphemes,
            special_toks,
            special_graphemes,
            para_codes,
        }
    }
}

struct Tokenizer<'a> {
    conf: &'a Conf,
    segment: Segment,
    state: TokenizingState,
    curr_tok_start: usize,
}

impl<'a> Tokenizer<'a> {
    fn tokenize(text: &str, conf: &'a Conf) -> Segment {
        // normalize whitespace for easier error reporting
        let text = WHITESPACE.replace_all(text, " ");
        let text = text.trim();
        let mut tokenizer = Tokenizer {
            conf,
            segment: Segment::new(text),
            state: Delim,
            curr_tok_start: 0,
        };
        for ig in text.grapheme_indices(true) {
            tokenizer.state = match tokenizer.state {
                Delim => tokenizer.leave_delim(ig),
                InsideWord => tokenizer.leave_inside_word(ig),
                InsideSpecial => tokenizer.leave_inside_special(ig),
                OpenBracket => tokenizer.leave_open_bracket(ig),
                CloseBracket => tokenizer.leave_close_bracket(ig),
            }
        }
        tokenizer.segment
    }

    fn leave_delim(&mut self, (index, grapheme): (usize, &str)) -> TokenizingState {
        match grapheme {
            "[" => {
                self.segment.push_token(OpenSquare, index, index + 1);
                OpenBracket
            }
            // "<" =>
            _ if self.conf.special_graphemes.contains(grapheme) => {
                self.curr_tok_start = index;
                InsideSpecial
            }
            // TODO: probably get rid of this, it can't occur since we normalize whitespace
            // _ if chr.is_whitespace() => Whitespace,
            _ if self.conf.word_graphemes.contains(grapheme) => {
                self.curr_tok_start = index;
                InsideWord
            }
            _ => {
                self.segment
                    .push_token(UnexpectedGrapheme, index, index + grapheme.len());
                Delim
            }
        }
    }

    fn leave_inside_word(&mut self, (index, grapheme): (usize, &str)) -> TokenizingState {
        if self.conf.word_graphemes.contains(grapheme) {
            return InsideWord;
        }
        self.segment.push_token(Word, self.curr_tok_start, index);
        match grapheme {
            "]" => {
                self.segment.push_token(CloseSquare, index, index + 1);
                CloseBracket
            }
            _ if self.conf.delim_graphemes.contains(grapheme) => Delim,
            _ => {
                self.segment
                    .push_token(UnexpectedGrapheme, index, index + grapheme.len());
                Delim
            }
        }
    }

    fn leave_inside_special(&mut self, (index, grapheme): (usize, &str)) -> TokenizingState {
        if self.conf.special_graphemes.contains(grapheme) {
            return InsideSpecial;
        }
        self.segment.push_token(Special, self.curr_tok_start, index);
        match grapheme {
            "]" => {
                self.segment.push_token(CloseSquare, index, index + 1);
                CloseBracket
            }
            _ if self.conf.delim_graphemes.contains(grapheme) => Delim,
            _ => {
                self.segment
                    .push_token(UnexpectedGrapheme, index, index + grapheme.len());
                Delim
            }
        }
    }

    fn leave_open_bracket(&mut self, (index, grapheme): (usize, &str)) -> TokenizingState {
        match grapheme {
            _ if self.conf.special_graphemes.contains(grapheme) => {
                self.curr_tok_start = index;
                InsideSpecial
            }
            _ if self.conf.word_graphemes.contains(grapheme) => {
                self.curr_tok_start = index;
                InsideWord
            }
            _ => {
                self.segment
                    .push_token(UnexpectedGrapheme, index, index + grapheme.len());
                Delim
            }
        }
    }

    fn leave_close_bracket(&mut self, (index, grapheme): (usize, &str)) -> TokenizingState {
        match grapheme {
            _ if self.conf.delim_graphemes.contains(grapheme) => Delim,
            _ => {
                self.segment
                    .push_token(UnexpectedGrapheme, index, index + grapheme.len());
                InsideWord
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer() {
        let conf1 = Conf::new("fočbarzqux", " ", "@", "");
        let conf2 = Conf::new("fočbarzqux", " ", "@", "");
        let seg = Tokenizer::tokenize("foo [čar @ baz] qux", &conf1);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [čar @ baz] qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [bar@ baz] qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[bar @ baz] qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo [bar @ baz]qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[bar @ baz ]qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        let seg = Tokenizer::tokenize("foo[ bar @ baz ]qux", &conf2);
        seg.debug();
        eprintln!("=======================================================");
        assert!(false);
    }
}
