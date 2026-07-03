use crate::core::context::{Context, HasContext};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::{ParseResult, Success};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

/// Collapses the delegate's matched span into the underlying `String`, discarding its value.
/// This is dart-petitparser's `FlattenParser` (constructed via `.flatten()`) — renamed here
/// since "flatten" reads as collapsing a nested structure, whereas this returns the raw
/// consumed input text. See [`crate::parser::ext::ParserExt::input`] for the constructor, and
/// its `.flatten()`/`.flatten_with_message()` aliases for anyone coming from dart.
#[derive(Clone, Debug)]
pub struct InputParser<T: Debug> {
    pub delegate: Rc<dyn Parser<T>>,
    pub message: Option<String>,
    pub delegate_type: PhantomData<T>,
}

impl<T: Debug + 'static> HasChildren for InputParser<T> {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![self.delegate.clone()]
    }

    fn is_input(&self) -> bool {
        true
    }

    fn input_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl<T: Debug + 'static> Parser<String> for InputParser<T> {
    fn parse_on(&self, context: &Context) -> ParseResult<String> {
        match &self.message {
            Some(m) => {
                let position = self.delegate.fast_parse_on(context);
                match position {
                    None => context.failure(m.clone()),
                    Some(pos) => context.success_with_position(
                        context.buffer()[context.position()..pos].iter().collect(),
                        pos,
                    ),
                }
            }
            None => {
                let result = self.delegate.parse_on(context);
                match result {
                    Err(f) => Err(f),
                    Ok(s) => {
                        let substring = context.buffer()[context.position()..s.context.position]
                            .iter()
                            .collect::<String>();
                        Ok(Success {
                            context: s.context.clone(),
                            value: substring,
                        })
                    }
                }
            }
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        self.delegate.fast_parse_on(context)
    }
}
