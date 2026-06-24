use crate::parser::ext::ParserExt;
use crate::parser::misc::newline::newline;
use std::cmp::{max, min};
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<T> {
    pub value: T,
    pub buffer: Rc<[char]>,
    pub start: usize,
    pub end: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, buffer: Rc<[char]>, start: usize, end: usize) -> Self {
        Self {
            value,
            buffer,
            start,
            end,
        }
    }

    pub fn input(&self) -> &[char] {
        &self.buffer[self.start..self.end]
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }

    pub fn line(&self) -> usize {
        line_and_column_of(self.buffer.clone(), self.start).0
    }

    pub fn column(&self) -> usize {
        line_and_column_of(self.buffer.clone(), self.start).1
    }

    pub fn join(tokens: impl IntoIterator<Item = Token<T>>) -> Token<Vec<T>> {
        let mut iter = tokens.into_iter();
        let first_token = iter
            .next()
            .expect("Token::join requires at least one token");
        let buffer = first_token.buffer;
        let mut start: usize = first_token.start;
        let mut end: usize = first_token.end;
        let mut values: Vec<T> = vec![first_token.value];

        for tok in iter {
            assert!(
                tok.buffer == buffer,
                "Token::join requires all tokens use the same buffer"
            );
            start = min(start, tok.start);
            end = max(end, tok.end);
            values.push(tok.value);
        }
        Token {
            value: values,
            buffer,
            start,
            end,
        }
    }
}

impl<T: Display> Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Token[{}]: {}",
            position_string(self.buffer.clone(), self.start),
            self.value
        )
    }
}

pub fn line_and_column_of(buffer: Rc<[char]>, position: usize) -> (usize, usize) {
    let mut line: usize = 1;
    let mut offset: usize = 0;
    for token in newline().token().all_matches(buffer, 0, false) {
        if position < token.end {
            return (line, position - offset + 1);
        }
        line += 1;
        offset = token.end;
    }
    (line, position - offset + 1)
}

pub fn position_string(buffer: Rc<[char]>, position: usize) -> String {
    let (line, column) = line_and_column_of(buffer, position);
    format!("{}:{}", line, column)
}
