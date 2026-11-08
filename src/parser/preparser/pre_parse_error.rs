use std::fmt;

use crate::lexer::span::Span;

/// Everything that can go wrong while splitting and classifying the words
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreParseErrorKind {
    /// A Hangul word that is no keyword, type, verb or declared name, even cut before a particle
    UnknownIdentifier(String),
    /// A declared name that is another name of the same scope followed by a particle: 사 and 사과
    AmbiguousName(String),
}

/// A pre-parsing error and where it happened
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreParseError {
    pub kind: PreParseErrorKind,
    pub span: Span,
}

impl PreParseError {
    pub fn new(kind: PreParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for PreParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIdentifier(word) => {
                write!(f, "'{word}' is no keyword, type, verb or declared name")
            }
            Self::AmbiguousName(name) => {
                write!(
                    f,
                    "'{name}' can be read as another declared name followed by a particle"
                )
            }
        }
    }
}

impl fmt::Display for PreParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for PreParseError {}
