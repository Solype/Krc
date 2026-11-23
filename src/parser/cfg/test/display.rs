use std::collections::{HashMap, HashSet};

use crate::parser::cfg::rulesorganiser::firsts::{FirstDerivation, FirstDerivationSet};
use crate::parser::cfg::rulesorganiser::follows::{FollowDerivation, FollowDerivationSet};
use crate::parser::cfg::structs::NTerm;
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
    let firsts = FirstDerivationSet {
        set: HashMap::from([
            (
                NTerm::F,
                FirstDerivation(HashSet::from([ParserTokenKind::Name(String::new())])),
            ),
            (
                NTerm::E,
                FirstDerivation(HashSet::from([ParserTokenKind::Int(0)])),
            ),
        ]),
        nullable: HashSet::from([NTerm::TPrime, NTerm::EPrime]),
    };
    assert_eq!(
        firsts.to_string(),
        "E: {Int}\nF: {Name}\nnullable: {EPrime, TPrime}\n"
    );
}

#[test]
fn no_nullable() {
    assert_eq!(FirstDerivationSet::default().to_string(), "nullable: {}\n");
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

#[test]
fn an_automata_state_shows_its_items_then_its_sorted_transitions() {
    use crate::parser::cfg::parsing_table::{AutomataState, Item};
    use crate::parser::cfg::rules::TEST_RULES;
    use crate::parser::cfg::rulesorganiser::rulesorganiser::RuleOrganiser;
    use crate::parser::cfg::structs::ParserNode;

    let rules = RuleOrganiser::new(TEST_RULES);
    let mut state = AutomataState::from_items(vec![
        Item::new(0, ParserTokenKind::Eof),
        Item {
            rule: 6,
            dot: 1,
            lookahead: ParserTokenKind::Operator(Operator::Plus),
        },
    ]);
    state.nexts.insert(ParserNode::NTerm(NTerm::T), 3);
    state.nexts.insert(ParserNode::Term(ParserTokenKind::Name(String::new())), 1);
    assert_eq!(
        state.display(&rules).to_string(),
        "[E → • T EPrime, Eof]\n\
         [F → Name •, Operator(Plus)]\n\
         Name → 1\n\
         T → 3\n"
    );
}
