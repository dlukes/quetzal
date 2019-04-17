use crate::tokenizer::TokenKind;

struct Node;

struct Mistake;

struct Segment {
    source: String,
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

enum ParsingState {
    Whitespace,
    NonWhitespace,
    OpenRound,
    CloseRound,
    OpenSquare,
    CloseSquare,
    OpenAngle,
    CloseAngle,
    AngleParam,
    Invalid,
}

struct Parser {
    current: usize,
    state: ParsingState,
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

impl Parser {
    fn new() -> Self {
        Self {
            current: 0,
            state: ParsingState::Whitespace,
            nodes: vec![],
            mistakes: vec![],
        }
    }

    fn parse(&mut self, segment: crate::Segment) -> Segment {
        self.current = 0;
        let tokens_length = segment.tokens.len();
        while self.current < tokens_length {
            self.state = match &self.state {
                ParsingState::Whitespace => self.leave_whitespace(&segment),
                ParsingState::NonWhitespace => self.leave_non_whitespace(&segment),
            }
        }
        // while let Some(node) = self.parse_node(&segment) {
        //     nodes.push(node);
        // }
        // Segment {
        //     source: segment.source,
        //     nodes,
        //     mistakes,
        // }
        unimplemented!()
    }

    fn leave_whitespace(&mut self, segment: &crate::Segment) -> ParsingState {
        let token = &segment.tokens[self.current];
        match token.kind {
            // shouldn't happen since we normalize whitespace but for good measure
            TokenKind::Whitespace => ParsingState::Whitespace,
            TokenKind::NonWhitespace => {
                self.consume_non_whitespace(token, segment);
                ParsingState::NonWhitespace
            }
            TokenKind::OpenRound => {
                self.consume_open_round(token);
                ParsingState::OpenRound
            }
            TokenKind::OpenSquare => {
                self.consume_open_square(token);
                ParsingState::OpenSquare
            }
            TokenKind::OpenAngle => {
                self.consume_open_angle(token);
                ParsingState::OpenAngle
            }
            _ => {
                self.mistakes.push(Mistake {});
                ParsingState::Invalid
            }
        }
    }

    fn leave_non_whitespace(&mut self, segment: &crate::Segment) -> ParsingState {
        let token = &segment.tokens[self.current];
        match token.kind {
            TokenKind::Whitespace => ParsingState::Whitespace,
            TokenKind::CloseRound => {
                self.consume_close_round(token);
                ParsingState::CloseRound
            }
            TokenKind::CloseSquare => {
                self.consume_close_square(token);
                ParsingState::CloseSquare
            }
            TokenKind::CloseAngle => {
                self.consume_close_angle(token);
                ParsingState::CloseAngle
            }
            _ => {
                self.mistakes.push(Mistake {});
                ParsingState::Invalid
            }
        }
    }
}

    // fn parse_node(&mut self, segment: &crate::Segment) -> Option<Node> {
    //     let token = segment.tokens.get(self.current)?;
    //     self.current += 1;
    //     match token.kind {
    //         Whitespace => self.parse_node(segment),
    //         NonWhitespace => self.parse_non_whitespace(token),
    //         OpenRound => ,
    //         CloseRound => ,
    //         OpenSquare => ,
    //         CloseSquare => ,
    //         OpenAngle => ,
    //         CloseAngle => ,
    //     }
    // }
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
