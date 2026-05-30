use crate::core::context::Context;
use crate::core::parser::Parser;
use crate::core::result::Failure;
use crate::core::result::ParseResult;

macro_rules! choice_impl {
    ($name:ident, $func:ident, $(($parser:ident , $value:ident)),+) => {
        #[derive(Clone, Debug)]
        pub struct $name<$($parser,)+> {
            $(pub $value: $parser,)+
            pub joiner: FailureJoiner,
        }

        impl <T, $($parser),+> Parser<T> for $name<$($parser),+>
        where
            $($parser: Parser<T>,)+
        {
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

        pub fn $func<$($parser),+>($($value: $parser,)+) -> $name<$($parser),+> {
            $name { $($value,)+ joiner: SELECT_FARTHEST }
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
