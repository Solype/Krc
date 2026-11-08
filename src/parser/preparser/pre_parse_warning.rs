use std::fmt;

use crate::lexer::span::Span;

/// Something suspicious the pre-parser accepts anyway
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreParseWarningKind {
    /// A declared verb in its dictionary form, but neither `(` nor `{` follows it: `바다() { } 바다`
    VerbWithoutParentheses(String),
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
        }
    }
}

impl fmt::Display for PreParseWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
