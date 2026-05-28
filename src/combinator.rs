use crate::context::{Context, ParseResult, Success};
use crate::core::Parser;
use crate::failure_joiner::{FailureJoiner, SELECT_FARTHEST};

macro_rules! impl_seq {
    ($name:ident, $func:ident, $(($parser:ident, $value:ident, $field:ident)),+$(,)?) => {
        #[derive(Clone, Debug)]
        pub struct $name<$($parser),+> {
            $(pub $field: $parser, )+
        }

        impl <$($value),+ , $($parser),+> Parser<($($value,)+)> for $name<$($parser),+>
        where
            $($parser: Parser<$value>,)+
        {
            fn parse_on(&self, context: &Context) -> ParseResult<($($value,)+)> {
                let mut ctx = context.clone();

                $(let success = self.$field.parse_on(&ctx)?;
                let $field = success.value;
                ctx = success.context;)+

                Ok(Success { context: ctx, value: ($($field,)+) })
            }
        }

        pub fn $func<$($parser),+>($($field: $parser,)+) -> $name<$($parser),+> {
            $name { $($field,)+ }
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

#[derive(Clone, Debug)]
pub struct Choice2<P1, P2> {
    pub p1: P1,
    pub p2: P2,
    pub joiner: FailureJoiner,
}

impl <T, P1, P2> Parser<T> for Choice2<P1, P2>
where
    P1: Parser<T>,
    P2: Parser<T>,
{
    fn parse_on(&self, context: &Context) -> ParseResult<T> {
        self.p1.parse_on(context).or_else(|f1| {
            self.p2.parse_on(context).map_err(|f2| (self.joiner)(f1, f2))
        })
    }
}


pub fn choice2<P1, P2>(p1: P1, p2: P2) -> Choice2<P1, P2> {
    Choice2 { p1, p2, joiner: SELECT_FARTHEST }
}

pub fn choice2_with_joiner<T, P1, P2>(p1: P1, p2: P2, joiner: FailureJoiner) -> Choice2<P1, P2>
where
    P1: Parser<T>,
    P2: Parser<T>,
{
    Choice2 { p1, p2, joiner }
}