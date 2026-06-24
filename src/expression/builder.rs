use std::{fmt::Debug, rc::Rc};

use crate::expression::group::choice_of;
use crate::{
    core::parser::Parser, expression::group::ExpressionGroup,
    parser::combinator::settable::SettableParser,
};

#[derive(Clone, Debug)]
pub struct ExpressionBuilder<T> {
    pub primitives: Vec<Rc<dyn Parser<T>>>,
    pub groups: Vec<ExpressionGroup<T>>,
    pub loopback: SettableParser<T>,
}

impl<T: Clone + Debug + 'static> Default for ExpressionBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Debug + 'static> ExpressionBuilder<T> {
    pub fn new() -> Self {
        ExpressionBuilder {
            primitives: Vec::new(),
            groups: Vec::new(),
            loopback: SettableParser::undefined(),
        }
    }

    pub fn primitive(&mut self, parser: impl Parser<T> + 'static) {
        self.primitives.push(Rc::new(parser));
    }

    pub fn group(&mut self) -> &mut ExpressionGroup<T> {
        self.groups
            .push(ExpressionGroup::new(self.loopback.borrow()));
        self.groups.last_mut().unwrap()
    }

    pub fn build(mut self) -> impl Parser<T> {
        assert!(
            !self.primitives.is_empty(),
            "At least one primitive parser required"
        );
        let parser = self
            .groups
            .into_iter()
            .fold(choice_of(self.primitives), |parser, group| {
                group.build(parser)
            });
        self.loopback.set(parser);
        self.loopback
    }
}
