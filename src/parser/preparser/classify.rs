use super::super::parser_token::{Keyword, ParserToken, ParserTokenKind, Type};
use crate::lexer::lexer_token::{LexerToken, LexerTokenKind, Operator, Punctuation};
use crate::lexer::span::Span;

use super::pre_parse_error::PreParseError;
use super::pre_parser::PreParser;
use super::words::{has_particule, is_bool, is_counter, is_keyword, is_particle, is_type};

/// One lexer token into parser tokens, before the declared names are known
impl PreParser<'_> {
    /// One lexer token gives one parser token, or two for a split word (정수를 → 정수 + 를);
    /// `prev` is the lexer token before it, if any
    pub(super) fn transform_token(
        &mut self,
        token: &LexerToken,
        prev: Option<&LexerTokenKind>,
    ) -> Result<Vec<ParserToken>, PreParseError> {
        let kind = match &token.kind {
            LexerTokenKind::HangulWord(word) => return self.hangul_word(word, token.span, prev),
            LexerTokenKind::LatinWord(name) => ParserTokenKind::Name(name.clone()),
            LexerTokenKind::Float(value) => ParserTokenKind::Float(*value),
            LexerTokenKind::Int(value) => ParserTokenKind::Int(*value),
            LexerTokenKind::Str(value) => ParserTokenKind::Str(value.clone()),
            LexerTokenKind::Punctuation(punctuation) => ParserTokenKind::Punctuation(*punctuation),
            LexerTokenKind::Operator(operator) => ParserTokenKind::Operator(*operator),
            LexerTokenKind::Eof => ParserTokenKind::Eof,
        };
        Ok(vec![ParserToken::new(kind, token.span)])
    }

    /// Classifies a Hangul word, in this order:
    /// 1. the whole word is a keyword, a type, a boolean, a counter before a type or a verb form:
    ///    반환하는, 정수, 참, 두 (정수), 더해서 (반환하는 is not 반환하 + 는)
    /// 2. after a number, a string, `)` or a Latin name, the whole word is a particle: 42를,
    ///    0이에요, printf를
    /// 3. a keyword, a type or a boolean followed by a particle: 정수를, 값이에요, 10개예요, 참이면
    /// 4. anything else is an identifier, left whole: 나를, 결과
    fn hangul_word(
        &mut self,
        word: &str,
        span: Span,
        prev: Option<&LexerTokenKind>,
    ) -> Result<Vec<ParserToken>, PreParseError> {
        let whole = |kind| Ok(vec![ParserToken::new(kind, span)]);

        if let Some(keyword) = is_keyword(word) {
            return whole(ParserTokenKind::Keyword(keyword));
        }
        if let Some(ty) = is_type(word) {
            return whole(ParserTokenKind::Type(ty));
        }
        if let Some(value) = is_bool(word) {
            return whole(ParserTokenKind::Bool(value));
        }
        if let Some(count) = is_counter(word)
            && self.type_follows()
        {
            return whole(ParserTokenKind::Counter(count));
        }
        if let Some((infinitive, ending)) = self.verbs.get(word) {
            return whole(ParserTokenKind::Verb {
                infinitive: infinitive.clone(),
                ending: *ending,
            });
        }

        let after_a_value = matches!(
            prev,
            Some(
                LexerTokenKind::Int(_)
                    | LexerTokenKind::Float(_)
                    | LexerTokenKind::Str(_)
                    | LexerTokenKind::Punctuation(Punctuation::RParen)
                    | LexerTokenKind::LatinWord(_)
            )
        );
        if after_a_value && let Some(particle) = is_particle(word) {
            return whole(ParserTokenKind::Particle(particle));
        }

        if let Some((stem, particle)) = has_particule(word) {
            let stem_kind = is_keyword(stem)
                .map(ParserTokenKind::Keyword)
                .or_else(|| is_type(stem).map(ParserTokenKind::Type))
                .or_else(|| is_bool(stem).map(ParserTokenKind::Bool));
            if let Some(stem_kind) = stem_kind {
                // byte offsets, like Span: 정수를 0..9 → 정수 0..6, 를 6..9
                let cut = span.start + stem.len();
                return Ok(vec![
                    ParserToken::new(stem_kind, Span::new(span.start, cut)),
                    ParserToken::new(
                        ParserTokenKind::Particle(particle),
                        Span::new(cut, span.end),
                    ),
                ]);
            }
        }

        // not split here: 결과 is a name, not 결 + 과, and only the declared names can tell
        whole(ParserTokenKind::Ident(word.to_string()))
    }

    /// Whether the next lexer token starts a type: 정수, or 짧은 / 부호 (없는) before it. A counter
    /// is only one before a type: 세 and 열 stay free as names
    fn type_follows(&self) -> bool {
        let LexerTokenKind::HangulWord(next) = &self.peek().kind else {
            return false;
        };
        is_type(next).is_some() || next == "짧은" || next == "부호"
    }

    /// `부호 없는` is two lexer tokens for one keyword: reads both, gives one `Unsigned` spanning
    /// them; None, reading nothing more, for anything else
    pub(super) fn merge_unsigned(&mut self, token: &LexerToken) -> Option<ParserToken> {
        let LexerTokenKind::HangulWord(first) = &token.kind else {
            return None;
        };
        let LexerTokenKind::HangulWord(second) = &self.peek().kind else {
            return None;
        };
        if first != "부호" || second != "없는" {
            return None;
        }
        let end = self.bump().span.end;
        Some(ParserToken::new(
            ParserTokenKind::Keyword(Keyword::Unsigned),
            Span::new(token.span.start, end),
        ))
    }

    /// `&` right after a type is `주소`: `정수&` → Type(Int) + Type(Address); since it becomes a
    /// type itself, `정수&&` is two addresses. Before the declared names, so that `정수& 가)`
    /// declares 가; anywhere else `&` stays the operator
    pub(super) fn amp_after_a_type(
        parsed: Vec<ParserToken>,
        last: Option<&ParserToken>,
    ) -> Vec<ParserToken> {
        match (parsed.as_slice(), last.map(|token| &token.kind)) {
            (
                [
                    ParserToken {
                        kind: ParserTokenKind::Operator(Operator::Amp),
                        span,
                    },
                ],
                Some(ParserTokenKind::Type(_)),
            ) => vec![ParserToken::new(
                ParserTokenKind::Type(Type::Address),
                *span,
            )],
            _ => parsed,
        }
    }
}
