use crate::core::context::Context;
use crate::core::kind::{ParserKind, PtrKey};
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::Failure;
use crate::core::result::ParseResult;
use std::fmt::Debug;
use std::rc::Rc;

macro_rules! choice_impl {
    ($name:ident, $func:ident, $(($parser:ident , $value:ident)),+) => {
        // Every alternative shares the same value type `T`, so the struct is generic over `T`
        // alone and each delegate is stored as `Rc<dyn Parser<T>>`.
        #[derive(Debug)]
        pub struct $name<T> {
            $(pub $value: Rc<dyn Parser<T>>,)+
            pub joiner: FailureJoiner,
        }

        // Manual `Clone` so the bound is just the cheap `Rc::clone` — `#[derive(Clone)]` would
        // spuriously require `T: Clone`.
        impl<T> Clone for $name<T> {
            fn clone(&self) -> Self {
                $name { $($value: self.$value.clone(),)+ joiner: self.joiner }
            }
        }

        impl<T: Debug + 'static> HasChildren for $name<T> {
            fn children(&self) -> Vec<Rc<dyn HasChildren>> {
                vec![$(self.$value.clone()),+]
            }

            fn kind(&self) -> ParserKind {
                ParserKind::Choice { joiner: self.joiner as PtrKey }
            }

            fn is_choice(&self) -> bool {
                true
            }
        }

        impl <T: Debug + 'static> Parser<T> for $name<T> {
            fn parse_on(&self, context: &Context) -> ParseResult<T> {
                let mut failure: Option<Failure> = None;
                $(
                    match self.$value.parse_on(context) {
                    Ok(s) => return Ok(s),
                        Err(f) => {
                            failure = Some(match failure {
                                None => f,
                                Some(prev) => (self.joiner)(prev, f),
                            });
                        }
                    }
                )+
                Err(failure.unwrap())
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn $func<T, $($parser),+>($($value: $parser,)+) -> $name<T>
        where
            T: 'static,
            $($parser: Parser<T> + 'static,)+
        {
            $name { $($value: Rc::new($value),)+ joiner: SELECT_FARTHEST }
        }
    };
}

choice_impl!(Choice2, choice2, (P1, p1), (P2, p2));
choice_impl!(Choice3, choice3, (P1, p1), (P2, p2), (P3, p3));
choice_impl!(Choice4, choice4, (P1, p1), (P2, p2), (P3, p3), (P4, p4));
choice_impl!(
    Choice5,
    choice5,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5)
);
choice_impl!(
    Choice6,
    choice6,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6)
);
choice_impl!(
    Choice7,
    choice7,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6),
    (P7, p7)
);
choice_impl!(
    Choice8,
    choice8,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6),
    (P7, p7),
    (P8, p8)
);
choice_impl!(
    Choice9,
    choice9,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6),
    (P7, p7),
    (P8, p8),
    (P9, p9)
);
choice_impl!(
    Choice10,
    choice10,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6),
    (P7, p7),
    (P8, p8),
    (P9, p9),
    (P10, p10)
);
choice_impl!(
    Choice11,
    choice11,
    (P1, p1),
    (P2, p2),
    (P3, p3),
    (P4, p4),
    (P5, p5),
    (P6, p6),
    (P7, p7),
    (P8, p8),
    (P9, p9),
    (P10, p10),
    (P11, p11)
);

pub type FailureJoiner = fn(Failure, Failure) -> Failure;

pub static SELECT_FIRST: FailureJoiner = |failure1, _| failure1;
pub static SELECT_SECOND: FailureJoiner = |_, failure2| failure2;
pub static SELECT_FARTHEST: FailureJoiner = |failure1, failure2| {
    if failure1.context.position <= failure2.context.position {
        failure2
    } else {
        failure1
    }
};
pub static SELECT_FARTHEST_JOINED: FailureJoiner = |failure1, failure2| {
    if failure1.context.position < failure2.context.position {
        failure2
    } else if failure2.context.position < failure1.context.position {
        failure1
    } else {
        Failure {
            context: failure1.context,
            message: format!("{} OR {}", failure1.message, failure2.message),
        }
    }
};
