use std::collections::{HashMap, HashSet};

use super::firsts::{FirstDerivation, FirstDerivationSet};
use super::follows::{FollowDerivation, FollowDerivationSet};
use super::structs::NTerm;
use crate::lexer::lexer_token::{Operator, Punctuation};
use crate::parser::parser_token::{Ending, ParserTokenKind};

#[test]
fn a_set_is_sorted_and_hides_placeholder_values() {
    let first = FirstDerivation(HashSet::from([
        ParserTokenKind::Punctuation(Punctuation::LParen),
        ParserTokenKind::Verb {
            infinitive: String::new(),
            ending: Ending::Seo,
        },
        ParserTokenKind::Name(String::new()),
    ]));
    assert_eq!(first.to_string(), "{Name, Punctuation(LParen), Verb(Seo)}");
}

#[test]
fn an_empty_set() {
    assert_eq!(FollowDerivation(HashSet::new()).to_string(), "{}");
}

#[test]
fn first_sets_follow_the_order_of_nterm() {
    let firsts = FirstDerivationSet(HashMap::from([
        (
            NTerm::F,
            FirstDerivation(HashSet::from([ParserTokenKind::Name(String::new())])),
        ),
        (
            NTerm::E,
            FirstDerivation(HashSet::from([ParserTokenKind::Int(0)])),
        ),
    ]));
    assert_eq!(firsts.to_string(), "E: {Int}\nF: {Name}\n");
}

#[test]
fn follow_sets_follow_the_order_of_nterm() {
    let follows = FollowDerivationSet(HashMap::from([
        (
            NTerm::TPrime,
            FollowDerivation(HashSet::from([ParserTokenKind::Operator(Operator::Plus)])),
        ),
        (
            NTerm::E,
            FollowDerivation(HashSet::from([
                ParserTokenKind::Eof,
                ParserTokenKind::Punctuation(Punctuation::RParen),
            ])),
        ),
    ]));
    assert_eq!(
        follows.to_string(),
        "E: {Eof, Punctuation(RParen)}\nTPrime: {Operator(Plus)}\n"
    );
}
