use std::collections::HashMap;

use super::super::parser_token::{Ending, ParserToken, ParserTokenKind};
use crate::lexer::lexer_token::{LexerToken, LexerTokenKind};

use super::pre_parse_error::{PreParseError, PreParseErrorKind};
use super::pre_parse_warning::PreParseWarning;
use super::words::{BUILTIN_VERBS, verb_forms};

/// Splits and classifies the lexer tokens (나를 → 나 + 를); `tokens` must end with `Eof`,
/// as `tokenize` returns them
pub fn pre_parse(tokens: &[LexerToken]) -> Result<PreParsed, PreParseError> {
    PreParser::new(tokens).run()
}

/// The pre-parser output: the tokens, and what looked suspicious on the way
pub struct PreParsed {
    pub tokens: Vec<ParserToken>,
    pub warnings: Vec<PreParseWarning>,
}

/// Reading position in the lexer tokens. Its methods are split by task: one word in
/// `classify.rs`, the verbs in 다 in `verbs.rs`, the declared names in `names.rs`
pub(super) struct PreParser<'a> {
    pub(super) tokens: &'a [LexerToken],
    /// Index of the current token, never past `Eof`
    pub(super) pos: usize,
    /// Every form of the known verbs → (infinitive, ending): 더해서 → (더하다, Seo)
    pub(super) verbs: HashMap<String, (String, Ending)>,
}

impl<'a> PreParser<'a> {
    fn new(tokens: &'a [LexerToken]) -> Self {
        let mut verbs = HashMap::new();
        for infinitive in BUILTIN_VERBS
            .iter()
            .map(|verb| verb.to_string())
            .chain(Self::declared_verbs(tokens))
        {
            for (form, ending) in verb_forms(&infinitive).into_iter().flatten() {
                verbs.insert(form, (infinitive.clone(), ending));
            }
        }
        Self {
            tokens,
            pos: 0,
            verbs,
        }
    }

    /// The current token, without moving
    pub(super) fn peek(&self) -> &'a LexerToken {
        &self.tokens[self.pos]
    }

    /// The current token, then moves to the next one (stays on `Eof`)
    pub(super) fn bump(&mut self) -> &'a LexerToken {
        let token = self.peek();
        if token.kind != LexerTokenKind::Eof {
            self.pos += 1;
        }
        token
    }

    fn run(mut self) -> Result<PreParsed, PreParseError> {
        let mut tokens = Vec::new();
        let mut warnings = Vec::new();
        let mut prev = None;
        loop {
            let token = self.bump();
            let parsed = match self.merge_unsigned(token) {
                Some(unsigned) => vec![unsigned],
                None => self.transform_token(token, prev)?,
            };
            let parsed = Self::amp_after_a_type(parsed, tokens.last());
            if let Some(warning) = self.check_parentheses(&parsed) {
                warnings.push(warning);
            }
            tokens.extend(parsed);
            if token.kind == LexerTokenKind::Eof {
                break;
            }
            // the last lexer token read: 없는 after a merged 부호 없는
            prev = Some(&self.tokens[self.pos - 1].kind);
        }
        let names = Self::declared_names(&tokens)?;
        for (i, replacement) in names.splits.into_iter().rev() {
            tokens.splice(i..=i, replacement);
        }
        let tokens = Self::tokenize_names(tokens, &names.names);
        // every word is classified by now: an `Ident` left is a name never declared
        if let Some(err) = tokens.iter().find_map(|token| match &token.kind {
            ParserTokenKind::Ident(word) => Some(PreParseError::new(
                PreParseErrorKind::UnknownIdentifier(word.clone()),
                token.span,
            )),
            _ => None,
        }) {
            return Err(err);
        }

        Ok(PreParsed { tokens, warnings })
    }
}
