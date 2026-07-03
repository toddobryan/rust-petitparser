use crate::core::context::{Context, HasContext};
use crate::core::kind::ParserKind;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct PossessiveRepeatingParser<T> {
    pub delegate: Rc<dyn Parser<T>>,
    pub min: usize,
    pub max: Option<usize>,
}

impl<T: Debug + 'static> HasChildren for PossessiveRepeatingParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn kind(&self) -> ParserKind {
        ParserKind::PossessiveRepeating {
            min: self.min,
            max: self.max,
        }
    }

    fn is_repeating(&self) -> bool {
        true
    }

    fn is_possessive_repeating(&self) -> bool {
        true
    }

    fn is_directly_nullable(&self) -> bool {
        self.min == 0
    }

    fn repeating_min(&self) -> Option<usize> {
        Some(self.min)
    }
}

impl<T> Parser<Vec<T>> for PossessiveRepeatingParser<T>
where
    T: Debug + 'static,
{
    fn parse_on(&self, context: &Context) -> ParseResult<Vec<T>> {
        let mut elements: Vec<T> = vec![];
        let mut ctx: Context = context.clone();

        while elements.len() < self.min {
            let result = self.delegate.parse_on(&ctx)?;
            assert!(
                ctx.position < result.context.position,
                "{:?} must consume input",
                self.delegate
            );
            elements.push(result.value);
            ctx = result.context;
        }
        while self.max.is_none() || elements.len() < self.max.unwrap() {
            let result = self.delegate.parse_on(&ctx);
            match result {
                Ok(s) => {
                    assert!(
                        ctx.position < s.context.position,
                        "{:?} must consume input",
                        self.delegate
                    );
                    elements.push(s.value);
                    ctx = s.context;
                }
                Err(_) => break,
            }
        }
        ctx.success(elements)
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let mut count: usize = 0;
        let mut current = context.position;
        while count < self.min {
            let new_pos: usize = self
                .delegate
                .fast_parse_on(&context.with_position(current))?;
            assert!(current < new_pos, "{:?} must consume input", self.delegate);
            count += 1;
            current = new_pos;
        }
        while self.max.is_none() || count < self.max.unwrap() {
            let result: Option<usize> =
                self.delegate.fast_parse_on(&context.with_position(current));
            match result {
                None => break,
                Some(new_pos) => {
                    assert!(current < new_pos, "{:?} must consume input", self.delegate);
                    count += 1;
                    current = new_pos;
                }
            }
        }
        Some(current)
    }
}
