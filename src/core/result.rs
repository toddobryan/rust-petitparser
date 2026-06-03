use crate::core::context::{Context, HasContext};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub type ParseResult<T> = Result<Success<T>, Failure>;

#[derive(Clone)]
pub struct Success<T> {
    pub context: Context,
    pub value: T,
}

impl<T> Display for Success<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Success[{}={}]: {:?}",
            self.position(),
            self.to_position_string(),
            self.value
        )
    }
}

impl<T> Debug for Success<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T> HasContext for Success<T> {
    fn buffer(&self) -> Rc<[char]> {
        self.context.buffer()
    }

    fn position(&self) -> usize {
        self.context.position()
    }
}

#[derive(Clone)]
pub struct Failure {
    pub context: Context,
    pub message: String,
}

impl HasContext for Failure {
    fn buffer(&self) -> Rc<[char]> {
        self.context.buffer()
    }

    fn position(&self) -> usize {
        self.context.position()
    }
}

impl Display for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failure[{}={}]: {}",
            self.position(),
            self.to_position_string(),
            self.message
        )
    }
}

impl Debug for Failure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
