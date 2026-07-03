use regex::{Captures, Match, Regex};

use crate::core::{
    context::{Context, HasContext},
    kind::ParserKind,
    parser::{HasChildren, Parser},
    result::{Failure, ParseResult, Success},
};
use std::rc::Rc;

pub fn regex(pattern: &str) -> RegexParser {
    let regex = Regex::new(pattern).unwrap();
    RegexParser {
        regex,
        message: format!("regex {:?} to match", pattern),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegexMatch {
    pub text: String,
    pub start: usize, // char- (not byte-) indexed
    pub end: usize,   // char- (not byte-) indexed
    pub groups: Vec<Option<String>>,
}

#[derive(Clone, Debug)]
pub struct RegexParser {
    pub regex: Regex,
    pub message: String,
}

impl RegexParser {
    fn byte_pos<'a>(&'a self, context: &'a Context) -> usize {
        context
            .text
            .char_indices()
            .nth(context.position)
            .map(|(b, _)| b)
            .unwrap_or(context.text.len())
    }
}

impl HasChildren for RegexParser {
    fn children(&self) -> Vec<Rc<dyn HasChildren>> {
        vec![]
    }

    fn kind(&self) -> ParserKind<'_> {
        ParserKind::Regex {
            pattern: self.regex.as_str(),
            message: &self.message,
        }
    }
}

impl Parser<RegexMatch> for RegexParser {
    fn parse_on(&self, context: &Context) -> ParseResult<RegexMatch> {
        let failure = Failure {
            context: context.clone(),
            message: self.message.clone(),
        };
        let byte_pos = self.byte_pos(context);
        let captures: Captures = self
            .regex
            .captures_at(context.text.as_str(), byte_pos)
            .ok_or(failure.clone())?;
        let overall_match = captures.get_match();
        if overall_match.start() == byte_pos {
            let text_ref = &context.text[byte_pos..overall_match.end()];
            let length = text_ref.chars().count();
            Ok(Success {
                value: RegexMatch {
                    text: text_ref.to_string(),
                    start: context.position,
                    end: context.position + length,
                    groups: captures
                        .iter()
                        .skip(1)
                        .map(|om: Option<Match>| om.map(|m: Match| m.as_str().to_string()))
                        .collect(),
                },
                context: context.with_position(context.position + length),
            })
        } else {
            Err(failure)
        }
    }

    fn fast_parse_on(&self, context: &Context) -> Option<usize> {
        let byte_pos = self.byte_pos(context);
        let first_match = self.regex.find_at(context.text.as_str(), byte_pos)?;
        if first_match.start() == byte_pos {
            let length = &context.text[byte_pos..first_match.end()].chars().count();
            Some(context.position + length)
        } else {
            None
        }
    }
}
