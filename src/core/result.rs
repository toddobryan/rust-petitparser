use crate::core::context::Context;
use std::fmt::{Display, Formatter};

pub type ParseResult<T> = Result<Success<T>, Failure>;

#[derive(Clone, Debug)]
pub struct Success<T> {
    pub context: Context,
    pub value: T,
}

#[derive(Clone, Debug)]
pub struct Failure {
    pub context: Context,
    pub message: String,
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failure[{}]: {}",
            self.context.to_position_string(),
            self.message
        )
    }
}
