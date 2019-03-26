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
