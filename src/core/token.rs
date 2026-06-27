use crate::core::context::{Context, HasContext};
use crate::parser::ext::ParserExt;
use crate::parser::misc::newline::newline;
use std::cmp::{max, min};
use std::fmt::{Debug, Display};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<T> {
    pub value: T,
    pub context: Context,
    pub end: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, context: Context, end: usize) -> Self {
        Self {
            value,
            context,
            end,
        }
    }

    pub fn start(&self) -> usize {
        self.context.position
    }

    pub fn input(&self) -> &[char] {
        &self.context.buffer[self.context.position..self.end]
    }

    pub fn length(&self) -> usize {
        self.end - self.context.position
    }

    pub fn line(&self) -> usize {
        line_and_column_of(&self.context).line
    }

    pub fn column(&self) -> usize {
        line_and_column_of(&self.context).column
    }

    pub fn join(tokens: impl IntoIterator<Item = Token<T>>) -> Token<Vec<T>> {
        let mut iter = tokens.into_iter();
        let first_token = iter
            .next()
            .expect("Token::join requires at least one token");
        let buffer = first_token.context.buffer.clone();
        let mut start: usize = first_token.context.position;
        let mut end: usize = first_token.end;
        let mut values: Vec<T> = vec![first_token.value];

        for tok in iter {
            assert!(
                tok.context.buffer == buffer,
                "Token::join requires all tokens use the same buffer"
            );
            start = min(start, tok.context.position);
            end = max(end, tok.end);
            values.push(tok.value);
        }
        Token {
            value: values,
            context: first_token.context.with_position(start),
            end,
        }
    }
}

impl<T: Display> Display for Token<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Token[{}]: {}",
            position_string(&self.context),
            self.value
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLocation {
    pub line: usize,
    pub column: usize,
}

pub fn line_and_column_of(context: &Context) -> TextLocation {
    let mut line: usize = 1;
    let mut offset: usize = 0;
    let position: usize = context.position;
    for token in newline()
        .token()
        .all_matches(context.with_position(0), false)
    {
        if position < token.end {
            return TextLocation {
                line,
                column: position - offset + 1,
            };
        }
        line += 1;
        offset = token.end;
    }
    TextLocation {
        line,
        column: position - offset + 1,
    }
}

pub fn position_string(context: &Context) -> String {
    let loc = line_and_column_of(context);
    format!("{}:{}", loc.line, loc.column)
}
