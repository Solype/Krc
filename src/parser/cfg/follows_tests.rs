//! FOLLOW of `TEST_RULES` (start symbol `E`), the values of the course:
//! FOLLOW(E) = FOLLOW(E′) = { ), $ }, FOLLOW(T) = FOLLOW(T′) = { +, ), $ }, FOLLOW(F) = { *, +, ), $ }
use std::collections::HashSet;

use super::firsts::FirstPairBuilder;
use super::follows::FollowsDerivationBuilder;
use super::rules::TEST_RULES;
use super::structs::NTerm;
use crate::lexer::lexer_token::{Operator, Punctuation};
use crate::parser::parser_token::ParserTokenKind;

fn follow_of(nterm: NTerm) -> HashSet<ParserTokenKind> {
    let firsts = FirstPairBuilder::firsts(TEST_RULES);
    let follows = FollowsDerivationBuilder::follows(TEST_RULES, &firsts, NTerm::E);
    println!("FIRST:\n{firsts}FOLLOW:\n{follows}");
    follows.0.get(&nterm).map(|follow| follow.0.clone()).unwrap_or_default()
}

const EOF: ParserTokenKind = ParserTokenKind::Eof;
const R_PAREN: ParserTokenKind = ParserTokenKind::Punctuation(Punctuation::RParen);
const PLUS: ParserTokenKind = ParserTokenKind::Operator(Operator::Plus);
const STAR: ParserTokenKind = ParserTokenKind::Operator(Operator::Star);

/// `$` because E is the start symbol, `)` from F → ( E )
#[test]
fn follow_of_e() {
    assert_eq!(follow_of(NTerm::E), HashSet::from([R_PAREN, EOF]));
}

/// E → T E′ : E′ ends E, so it gets FOLLOW(E)
#[test]
fn follow_of_e_prime() {
    assert_eq!(follow_of(NTerm::EPrime), HashSet::from([R_PAREN, EOF]));
}

/// E → T E′ : FIRST(E′) without ε, and FOLLOW(E) because E′ can be ε
#[test]
fn follow_of_t() {
    assert_eq!(follow_of(NTerm::T), HashSet::from([PLUS, R_PAREN, EOF]));
}

/// T → F T′ : T′ ends T, so it gets FOLLOW(T)
#[test]
fn follow_of_t_prime() {
    assert_eq!(follow_of(NTerm::TPrime), HashSet::from([PLUS, R_PAREN, EOF]));
}

/// T → F T′ : FIRST(T′) without ε, and FOLLOW(T) because T′ can be ε
#[test]
fn follow_of_f() {
    assert_eq!(follow_of(NTerm::F), HashSet::from([STAR, PLUS, R_PAREN, EOF]));
}
