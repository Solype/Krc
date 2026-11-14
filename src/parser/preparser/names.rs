use std::collections::{BTreeMap, HashSet};

use super::super::parser_token::{Ending, Keyword, ParserToken, ParserTokenKind, Particle};
use crate::lexer::lexer_token::Punctuation;
use crate::lexer::span::Span;

use super::pre_parse_error::{PreParseError, PreParseErrorKind};
use super::pre_parse_warning::{PreParseWarning, PreParseWarningKind};
use super::pre_parser::PreParser;
use super::words::{expected_form, has_particule, is_name_particle};

/// The declared names: which `Ident`s declare one, then every `Ident` cut into a name and its
/// particle
impl PreParser<'_> {
    fn cutted_name<'w>(word: &'w str, names: &HashSet<&str>) -> Option<(&'w str, Particle)> {
        word.char_indices()
            .rev()
            .filter(|(cut, _)| *cut > 0)
            .map(|(cut, _)| word.split_at(cut))
            .find_map(|(name, rest)| {
                let particle = is_name_particle(rest)?;
                names.contains(name).then_some((name, particle))
            })
    }
    /// Replaces every `Ident` left that is a declared name, alone or followed by a particle a name
    /// can carry: 나 → Name(나), 나를 → Name(나) + Particle(Object). When several names fit, the
    /// longest wins (가나를 → 가나 + 를, not 가 + 나를); any other `Ident` stays
    pub(super) fn tokenize_names(
        &mut self,
        tokens: Vec<ParserToken>,
        names: &[String],
    ) -> Vec<ParserToken> {
        let names: HashSet<&str> = names.iter().map(String::as_str).collect();
        let mut out = Vec::with_capacity(tokens.len());

        for token in tokens {
            let ParserTokenKind::Ident(word) = &token.kind else {
                out.push(token);
                continue;
            };
            if names.contains(word.as_str()) {
                out.push(ParserToken::new(
                    ParserTokenKind::Name(word.clone()),
                    token.span,
                ));
                continue;
            }
            // from the longest name down: 가나를 tries 가나 before 가
            let split = Self::cutted_name(word, &names);
            match split {
                Some((name, particle)) => out.extend(Self::split_name(name, particle, token.span)),
                None => out.push(token),
            }
        }
        out
    }

    fn is_word_ambiguous(names: &Vec<String>, to_test: &str) -> bool {
        for name in names {
            if !name.starts_with(to_test) && !to_test.starts_with(name) {
                continue;
            }
            let suffix = if name.len() > to_test.len() {
                name.strip_prefix(to_test)
            } else {
                to_test.strip_prefix(name)
            };

            if let Some(s) = suffix
                && is_name_particle(s).is_some()
            {
                return true;
            }
        }
        false
    }

    /// Every declared variable and parameter name, from the pre-parsed tokens (names still `Ident`)
    pub(super) fn declared_names(
        &mut self,
        tokens: &[ParserToken],
    ) -> Result<DeclaredNames, PreParseError> {
        let mut scan = NameScan::default();
        for (i, token) in tokens.iter().enumerate() {
            let ParserTokenKind::Ident(word) = &token.kind else {
                scan.track_blocks(&token.kind);
                continue;
            };
            let prev = i.checked_sub(1).map(|p| &tokens[p].kind);
            let next = tokens.get(i + 1).map(|t| &t.kind);
            if Self::is_parameter(prev, next) {
                scan.declare(word, token.span)?;
                continue;
            }
            self.declared_variable(&mut scan, i, word, token.span, prev)?;
        }
        Ok(scan.declared)
    }

    /// A parameter: `정수 가,` or `정수 나)`, the whole word is the name
    fn is_parameter(prev: Option<&ParserTokenKind>, next: Option<&ParserTokenKind>) -> bool {
        matches!(prev, Some(ParserTokenKind::Type(_)))
            && matches!(
                next,
                Some(ParserTokenKind::Punctuation(
                    Punctuation::Comma | Punctuation::RParen
                ))
            )
    }

    /// The `Ident` `word` (token `i`) that declares a name with its particle:
    /// variable: `결과는 0이에요`, 은/는 at the start of a statement;
    /// loop: `정수 칸을 0부터 10까지 세면서`, 을/를 after a type (`칸을` alone uses 칸);
    /// sentence declaration: `정수 가와 정수 나를 받아`, 와/과 or 을/를 after a type, or
    /// after a parameter with 와 (`두 정수 가와 나를`: the post-parser checks the count)
    fn declared_variable(
        &mut self,
        scan: &mut NameScan,
        i: usize,
        word: &str,
        span: Span,
        prev: Option<&ParserTokenKind>,
    ) -> Result<(), PreParseError> {
        let untyped = std::mem::take(&mut scan.untyped_next);
        let Some((name, particle)) = has_particule(word) else {
            return Ok(());
        };

        let declares = match particle {
            Particle::Topic => Self::starts_a_statement(prev),
            Particle::Object | Particle::With => {
                matches!(prev, Some(ParserTokenKind::Type(_))) || untyped
            }
            _ => false,
        };
        if !declares || name.is_empty() {
            return Ok(());
        }

        let expected = expected_form(name, &particle);
        if word != expected {
            self.warnings.push(PreParseWarning::new(
                PreParseWarningKind::BadBatchim {
                    found: word.to_string(),
                    expected,
                },
                span,
            ));
        }

        scan.declare(name, span)?;
        // a variable outside any function is global; a parameter at depth 0 is not
        if scan.depth == 0 && particle == Particle::Topic {
            scan.globals = scan.scope.len();
        }
        scan.declared
            .splits
            .insert(i, Self::split_name(name, particle, span));
        scan.untyped_next = particle == Particle::With;
        Ok(())
    }

    /// A declaring word cut after its name, each part with its own span
    /// (byte offsets, like Span: 결과는 0..9 → 결과 0..6, 는 6..9)
    fn split_name(name: &str, particle: Particle, span: Span) -> [ParserToken; 2] {
        let cut = span.start + name.len();
        [
            ParserToken::new(
                ParserTokenKind::Name(name.to_string()),
                Span::new(span.start, cut),
            ),
            ParserToken::new(
                ParserTokenKind::Particle(particle),
                Span::new(cut, span.end),
            ),
        ]
    }

    /// Whether the word after `prev` starts a statement: first word, after `{`, `}`, `.`, 고정된,
    /// or after a statement that ended in 요 (반환해요, 더해요, 이에요…)
    fn starts_a_statement(prev: Option<&ParserTokenKind>) -> bool {
        matches!(
            prev,
            None | Some(
                ParserTokenKind::Punctuation(
                    Punctuation::LBrace | Punctuation::RBrace | Punctuation::Dot
                ) | ParserTokenKind::Keyword(
                    Keyword::Const | Keyword::Return | Keyword::Stop | Keyword::Skip
                ) | ParserTokenKind::Verb {
                    ending: Ending::Yo,
                    ..
                } | ParserTokenKind::Particle(Particle::ItIs)
            )
        )
    }
}

/// The names declared by the `Ident`s of the pre-parsed tokens
#[derive(Debug, Default)]
pub(super) struct DeclaredNames {
    /// In the order of the file; a name declared twice appears twice
    pub(super) names: Vec<String>,
    /// Index of a declaring `Ident` → the tokens that replace it, only when it is split:
    /// 결과는 → [Name(결과), Particle(Topic)]; nothing for a parameter (가)
    pub(super) splits: BTreeMap<usize, [ParserToken; 2]>,
}

/// Where `declared_names` is in the tokens: what it found, and the names in scope
#[derive(Default)]
struct NameScan {
    declared: DeclaredNames,
    /// `두 정수 가와 나를`: after a parameter with 와, the next name may come without its type
    untyped_next: bool,
    /// The names a new one can conflict with: the globals, then those of the current function
    scope: Vec<String>,
    /// scope[..globals] are declared outside any function, and stay when one ends
    globals: usize,
    depth: usize,
}

impl NameScan {
    /// `{` and `}` open and close a block; a `(` at depth 0 starts a new header, whose
    /// parameters replace those of a header without a body (외부)
    fn track_blocks(&mut self, kind: &ParserTokenKind) {
        match kind {
            ParserTokenKind::Punctuation(Punctuation::LBrace) => self.depth += 1,
            ParserTokenKind::Punctuation(Punctuation::LParen) if self.depth == 0 => {
                self.scope.truncate(self.globals);
            }
            ParserTokenKind::Punctuation(Punctuation::RBrace) => {
                self.depth = self.depth.saturating_sub(1);
                // a function ends: its parameters and variables go, the globals stay
                if self.depth == 0 {
                    self.scope.truncate(self.globals);
                }
            }
            _ => {}
        }
    }

    /// Adds `name` to the declared names and to the scope; an error when it differs from a name
    /// of the scope by a particle only
    fn declare(&mut self, name: &str, span: Span) -> Result<(), PreParseError> {
        if PreParser::is_word_ambiguous(&self.scope, name) {
            return Err(PreParseError::new(
                PreParseErrorKind::AmbiguousName(name.to_string()),
                span,
            ));
        }
        self.declared.names.push(name.to_string());
        self.scope.push(name.to_string());
        Ok(())
    }
}
