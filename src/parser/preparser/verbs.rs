use super::super::parser_token::{Ending, ParserToken, ParserTokenKind};
use crate::lexer::lexer_token::{LexerToken, LexerTokenKind, Punctuation};

use super::pre_parse_warning::{PreParseWarning, PreParseWarningKind};
use super::pre_parser::PreParser;

/// The verbs in 다: which words of the file are verbs, and whether a definition follows them
impl PreParser<'_> {
    /// First pass: the infinitive of every verb of the file, so that a call before the
    /// definition is a verb too. 다 marks the functions: any Hangul word ending in 다 is a verb in
    /// its dictionary form. A sentence declaration gives its verb in -고, right after 받아 or with
    /// `반환` after it and before the `{`: `받아 비교하고`, `인사하고 정수를 반환` → 비교하다, 인사하다
    pub(super) fn declared_verbs(tokens: &[LexerToken]) -> impl Iterator<Item = String> {
        tokens.iter().enumerate().filter_map(|(i, token)| {
            let LexerTokenKind::HangulWord(word) = &token.kind else {
                return None;
            };
            if word.ends_with('다') {
                return Some(word.to_string());
            }
            let stem = word.strip_suffix('고')?;
            let after_receiving = i > 0
                && matches!(&tokens[i - 1].kind, LexerTokenKind::HangulWord(prev) if prev == "받아");
            (after_receiving || Self::gives_before_block(&tokens[i + 1..]))
                .then(|| format!("{stem}다"))
        })
    }

    /// Whether `반환` comes before the next `{` or `}`: the end of a sentence declaration
    fn gives_before_block(tokens: &[LexerToken]) -> bool {
        tokens
            .iter()
            .take_while(|token| {
                !matches!(
                    token.kind,
                    LexerTokenKind::Punctuation(Punctuation::LBrace | Punctuation::RBrace)
                )
            })
            .any(|token| matches!(&token.kind, LexerTokenKind::HangulWord(word) if word == "반환"))
    }

    /// A verb in its dictionary form must be followed by `(` or `{`: `더하다(…)` and `인사하다 {`
    /// are definitions, `바다` alone is probably a name ending in 다
    pub(super) fn check_parentheses(&mut self, parsed: &[ParserToken]) {
        let [token] = parsed else {
            return;
        };
        let ParserTokenKind::Verb {
            infinitive,
            ending: Ending::Da,
        } = &token.kind
        else {
            return;
        };
        let defined = matches!(
            self.peek().kind,
            LexerTokenKind::Punctuation(Punctuation::LParen | Punctuation::LBrace)
        );
        if !defined {
            self.warnings.push(PreParseWarning::new(
                PreParseWarningKind::VerbWithoutParentheses(infinitive.clone()),
                token.span,
            ));
        }
    }
}
