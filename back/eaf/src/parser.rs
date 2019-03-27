use crate::tokenizer::TokenKind::*;

struct Node;

struct Mistake;

struct Segment {
    source: String,
    nodes: Vec<Node>,
    mistakes: Vec<Mistake>,
}

struct Parser {
    current: usize,
}

impl Parser {
    fn new() -> Self {
        Self { current: 0 }
    }

    fn parse(&mut self, segment: crate::Segment) -> Segment {
        self.current = 0;
        let nodes = vec![];
        let mistakes = vec![];
        while let Some(node) = self.parse_node(&segment) {
            nodes.push(node);
        }
        Segment {
            source: segment.source,
            nodes,
            mistakes,
        }
    }

    fn parse_node(&mut self, segment: &crate::Segment) -> Option<Node> {
        let token = segment.tokens.get(self.current)?;
        self.current += 1;
        match token.kind {
            Whitespace => self.parse_node(segment),
            NonWhitespace => self.parse_non_whitespace(token),
            OpenRound => ,
            CloseRound => ,
            OpenSquare => ,
            CloseSquare => ,
            OpenAngle => ,
            CloseAngle => ,
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
