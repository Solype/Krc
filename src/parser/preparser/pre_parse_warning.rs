use std::fmt;

use crate::lexer::span::Span;

/// Something suspicious the pre-parser accepts anyway
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreParseWarningKind {
    /// A declared verb in its dictionary form, but neither `(` nor `{` follows it: `바다() { } 바다`
    VerbWithoutParentheses(String),
    /// A particle with two forms in the form of the other 받침: `found` is the word as written
    /// (끝는), `expected` the same word with the right form (끝은)
    BadBatchim { found: String, expected: String },
}

/// A pre-parsing warning and where it happened
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreParseWarning {
    pub kind: PreParseWarningKind,
    pub span: Span,
}

impl PreParseWarning {
    pub fn new(kind: PreParseWarningKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for PreParseWarningKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VerbWithoutParentheses(word) => write!(
                f,
                "'{word}' is a declared function, but no '(' or '{{' follows"
            ),
            Self::BadBatchim { found, expected } => write!(
                f,
                "'{found}': the particle does not fit the 받침 before it, write '{expected}'"
            ),
        }
    }
}

impl fmt::Display for PreParseWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
