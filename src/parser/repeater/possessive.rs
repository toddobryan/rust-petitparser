use std::fmt::Debug;
use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::ParseResult;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct PossessiveRepeatingParser<P> {
    pub delegate: P,
    pub min: usize,
    pub max: Option<usize>,
}

impl<T, P> Parser<Vec<T>> for PossessiveRepeatingParser<P>
where
    P: Parser<T>,
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
        ctx.success(elements, ctx.position)
    }

    fn fast_parse_on(&self, buffer: Rc<[char]>, position: usize) -> Option<usize> {
        let mut count: usize = 0;
        let mut current = position;
        while count < self.min {
            let new_pos: usize = self.delegate.fast_parse_on(buffer.clone(), current)?;
            assert!(current < new_pos, "{:?} must consume input", self.delegate);
            count += 1;
            current = new_pos;
        }
        while self.max.is_none() || count < self.max.unwrap() {
            let result: Option<usize> = self.delegate.fast_parse_on(buffer.clone(), current);
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
