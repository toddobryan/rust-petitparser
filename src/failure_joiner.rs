use crate::context::Failure;

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
            context: failure1.context ,
            message: format!("{} OR {}", failure1.message, failure2.message)}
    }
};