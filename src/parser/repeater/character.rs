use std::fmt::Debug;
use std::rc::Rc;

use crate::core::{
    context::{Context, HasContext},
    kind::{ParserKind, PtrKey},
    parser::{HasChildren, Parser},
    result::ParseResult,
};

#[derive(Clone)]
pub struct CharacterRepeatingParser {
    pub test: Rc<dyn Fn(char) -> bool>,
    pub description: String,
    pub message: Option<String>,
    pub min: usize,
    pub max: Option<usize>,
}

impl CharacterRepeatingParser {
    fn message_for(&self, found: Option<char>) -> String {
        if let Some(m) = &self.message {
            m.clone()
        } else {
            match found {
                Some(c) => format!("expected {}, but found '{}'", self.description, c),
                None => format!("expected {}, but reached end of input", self.description),
            }
        }
    }
}

impl Debug for CharacterRepeatingParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacterRepeatingParser")
            .field("test", &"<predicate function>")
            .field("description", &self.description)
            .field("message", &self.message)
            .field("min", &self.min)
            .field("max", &self.max)
            .finish()
    }
}

impl HasChildren for CharacterRepeatingParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::CharacterRepeating {
            test: Rc::as_ptr(&self.test) as PtrKey,
            message: self.message.clone(),
            min: self.min,
            max: self.max,
        }
    }

    fn is_char_repeating(&self) -> bool {
        true
    }

    fn is_directly_nullable(&self) -> bool {
        self.min == 0
    }
}

impl Parser<String> for CharacterRepeatingParser {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        let buffer = context.buffer();
        let start = context.position();
        let end = buffer.len();
        let mut current: usize = context.position;
        let mut count: usize = 0;

        while (self.max.is_none() || count < self.max.unwrap())
            && current < end
            && (self.test)(buffer[current])
        {
            current += 1;
            count += 1;
        }

        if count >= self.min {
            context.success_with_position(buffer[start..current].iter().collect(), current)
        } else {
            context.failure_with_position(self.message_for(buffer.get(current).copied()), current)
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let end = context.buffer.len();
        let mut current = context.position;
        let mut count = 0_usize;
        while (self.max.is_none() || count < self.max.unwrap())
            && current < end
            && (self.test)(context.buffer[current])
        {
            current += 1;
            count += 1;
        }
        if count >= self.min {
            Some(current)
        } else {
            None
        }
    }
}
