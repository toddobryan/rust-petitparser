use crate::core::context::Context;
use crate::core::parser::{HasChildren, Parser};
use crate::core::result::ParseResult;
use crate::core::result::Success;
use std::fmt::Debug;
use std::rc::Rc;

macro_rules! impl_seq {
    ($name:ident, $func:ident, $(($parser:ident, $value:ident, $field:ident)),+$(,)?) => {
        #[derive(Debug)]
        pub struct $name<$($value),+> {
            $(pub $field: Rc<dyn Parser<$value>>, )+
        }

        // Manual `Clone` so the bound is just the cheap `Rc::clone` — `#[derive(Clone)]` would
        // spuriously require every value type to be `Clone`.
        impl <$($value),+> Clone for $name<$($value),+> {
            fn clone(&self) -> Self {
                $name { $($field: self.$field.clone(),)+ }
            }
        }

        impl <$($value),+> HasChildren for $name<$($value),+>
        where
            $($value: Debug + 'static,)+
        {
            fn children(&self) -> Vec<Rc<dyn HasChildren>> {
                vec![$(self.$field.clone()),+]
            }
        }

        impl <$($value),+> Parser<($($value,)+)> for $name<$($value),+>
        where
            $($value: Debug + 'static,)+
        {
            fn parse_on(&self, context: &Context) -> ParseResult<($($value,)+)> {
                let mut ctx = context.clone();

                $(let success = self.$field.parse_on(&ctx)?;
                let $field = success.value;
                ctx = success.context;)+

                Ok(Success { context: ctx, value: ($($field,)+) })
            }
        }

        #[allow(clippy::too_many_arguments)]
        pub fn $func<$($parser),+, $($value),+>($($field: $parser,)+) -> $name<$($value),+>
        where
            $($value: 'static,)+
            $($parser: Parser<$value> + 'static,)+
        {
            $name { $($field: Rc::new($field),)+ }
        }
    };
}

impl_seq!(Seq2, seq2, (P1, T1, p1), (P2, T2, p2));
impl_seq!(Seq3, seq3, (P1, T1, p1), (P2, T2, p2), (P3, T3, p3));
impl_seq!(
    Seq4,
    seq4,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4)
);
impl_seq!(
    Seq5,
    seq5,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4),
    (P5, T5, p5)
);
impl_seq!(
    Seq6,
    seq6,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4),
    (P5, T5, p5),
    (P6, T6, p6)
);
impl_seq!(
    Seq7,
    seq7,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4),
    (P5, T5, p5),
    (P6, T6, p6),
    (P7, T7, p7)
);
impl_seq!(
    Seq8,
    seq8,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4),
    (P5, T5, p5),
    (P6, T6, p6),
    (P7, T7, p7),
    (P8, T8, p8)
);
impl_seq!(
    Seq9,
    seq9,
    (P1, T1, p1),
    (P2, T2, p2),
    (P3, T3, p3),
    (P4, T4, p4),
    (P5, T5, p5),
    (P6, T6, p6),
    (P7, T7, p7),
    (P8, T8, p8),
    (P9, T9, p9)
);
