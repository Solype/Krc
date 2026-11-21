use std::collections::HashSet;

use super::firsts::FirstPairBuilder;
use super::rules::TEST_RULES;
use super::structs::NTerm;
use crate::lexer::lexer_token::{Operator, Punctuation};
use crate::parser::parser_token::ParserTokenKind;

#[test]
fn firsts_of_test_rules() {
    let firsts = FirstPairBuilder::firsts(TEST_RULES);
    let id_or_paren = HashSet::from([
        ParserTokenKind::Name(String::new()),
        ParserTokenKind::Punctuation(Punctuation::LParen),
    ]);
    assert_eq!(firsts.set[&NTerm::E].0, id_or_paren);
    assert_eq!(firsts.set[&NTerm::T].0, id_or_paren);
    assert_eq!(firsts.set[&NTerm::F].0, id_or_paren);
    // ε is in `nullable`, not in the sets
    assert_eq!(
        firsts.set[&NTerm::EPrime].0,
        HashSet::from([ParserTokenKind::Operator(Operator::Plus)])
    );
    assert_eq!(
        firsts.set[&NTerm::TPrime].0,
        HashSet::from([ParserTokenKind::Operator(Operator::Star)])
    );
    assert_eq!(firsts.nullable, HashSet::from([NTerm::EPrime, NTerm::TPrime]));
}
