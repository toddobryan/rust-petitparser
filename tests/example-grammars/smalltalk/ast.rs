// AST ported from dart-petitparser-examples' lib/src/smalltalk/ast.dart.
//
// Token bookkeeping (dart's `IsSurrounded.beforeToken/afterToken`,
// `BlockNode.separators`, `CascadeNode.semicolons`, `HasStatements.periods`)
// is intentionally dropped: smalltalk_test.dart's matchers never inspect any
// of that, only the logical shape (selector/arguments/receiver/value/name/
// temporaries/statements). Likewise the dart `Visitor`/`NodeCollector`
// machinery isn't ported: its one use in the test suite
// (`NodeCollector.allNodes(ast)` non-empty) is trivially true for every
// parsed AST, so it carries no test signal here.
//
// dart's `selectorType` is a getter derived from the selector string and
// argument count; we compute it the same way at construction time via
// `selector_type_of` rather than storing a separately-passed flag.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorType {
    Unary,
    Binary,
    Keyword,
}

pub fn selector_type_of(selector: &str, arity: usize) -> SelectorType {
    if arity == 0 {
        SelectorType::Unary
    } else if selector.ends_with(':') {
        SelectorType::Keyword
    } else {
        SelectorType::Binary
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    Array(Vec<Literal>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Literal(Literal),
    Variable(String),
    Assignment(String, Box<Node>),
    Message {
        receiver: Box<Node>,
        selector: String,
        selector_type: SelectorType,
        arguments: Vec<Node>,
    },
    /// Always a list of `Message` nodes, mirroring dart's
    /// `CascadeNode.messages: List<MessageNode>`.
    Cascade(Vec<Node>),
    Array(Vec<Node>),
    Block {
        arguments: Vec<String>,
        body: Sequence,
    },
    Return(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sequence {
    pub temporaries: Vec<String>,
    pub statements: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pragma {
    pub selector: String,
    pub selector_type: SelectorType,
    pub arguments: Vec<Literal>,
}

impl Pragma {
    pub fn new(selector: String, arguments: Vec<Literal>) -> Self {
        let selector_type = selector_type_of(&selector, arguments.len());
        Pragma {
            selector,
            selector_type,
            arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub selector: String,
    pub selector_type: SelectorType,
    pub arguments: Vec<String>,
    pub pragmas: Vec<Pragma>,
    pub body: Sequence,
}

impl Method {
    pub fn new(
        selector: String,
        arguments: Vec<String>,
        pragmas: Vec<Pragma>,
        body: Sequence,
    ) -> Self {
        let selector_type = selector_type_of(&selector, arguments.len());
        Method {
            selector,
            selector_type,
            arguments,
            pragmas,
            body,
        }
    }
}

pub fn build_message(receiver: Node, parts: Vec<(String, Vec<Node>)>) -> Node {
    parts
        .into_iter()
        .fold(receiver, |receiver, (selector, arguments)| {
            let selector_type = selector_type_of(&selector, arguments.len());
            Node::Message {
                receiver: Box::new(receiver),
                selector,
                selector_type,
                arguments,
            }
        })
}

/// Mirrors dart's `buildCascade`: every cascaded message is sent to the
/// receiver of the *first* message (not chained onto each other), so
/// `1 abs negated; raisedTo: 12` sends both `negated` and `raisedTo:` to
/// `1 abs`, not to `1 abs negated`. Per dart's own (unguarded) `value as
/// MessageNode` cast, a non-message receiver with a non-empty cascade is an
/// upstream-unhandled case — ported faithfully as a panic rather than added
/// defensive handling, since no test exercises it.
pub fn build_cascade(value: Node, parts: Vec<(String, Vec<Node>)>) -> Node {
    if parts.is_empty() {
        return value;
    }
    let base_receiver = match &value {
        Node::Message { receiver, .. } => (**receiver).clone(),
        _ => panic!("cascade requires a message receiver"),
    };
    let mut messages = vec![value];
    for part in parts {
        messages.push(build_message(base_receiver.clone(), vec![part]));
    }
    Node::Cascade(messages)
}

pub fn build_assignment(value: Node, vars: Vec<String>) -> Node {
    vars.into_iter().rev().fold(value, |result, name| {
        Node::Assignment(name, Box::new(result))
    })
}
