use std::collections::HashSet;

use crate::parser::cfg::rulesorganiser::firsts::FirstPairBuilder;
use crate::parser::cfg::rules::TEST_RULES;
use crate::parser::cfg::structs::NTerm;
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

/// A → B C is nullable only when B and C both are, a rule with a terminal never is
#[test]
fn nullable_needs_every_symbol_nullable() {
    use crate::parser::cfg::structs::{ParserNode, ParserProduction};
    static RULES: &[ParserProduction] = &[
        // E → T F : F is not nullable, so E is not
        ParserProduction {
            left: NTerm::E,
            right: &[ParserNode::NTerm(NTerm::T), ParserNode::NTerm(NTerm::F)],
        },
        // EPrime → T TPrime : both nullable
        ParserProduction {
            left: NTerm::EPrime,
            right: &[ParserNode::NTerm(NTerm::T), ParserNode::NTerm(NTerm::TPrime)],
        },
        ParserProduction {
            left: NTerm::T,
            right: &[],
        },
        // TPrime → T : nullable through T
        ParserProduction {
            left: NTerm::TPrime,
            right: &[ParserNode::NTerm(NTerm::T)],
        },
        ParserProduction {
            left: NTerm::F,
            right: &[ParserNode::Term(ParserTokenKind::Eof)],
        },
    ];
    let mut builder = FirstPairBuilder::default();
    builder.get_nullable(RULES);
    assert_eq!(
        builder.nullable,
        HashSet::from([NTerm::T, NTerm::TPrime, NTerm::EPrime])
    );
}

/// E → T $, T → ε | Int : FIRST(E) reads past the nullable T, and E is not nullable
#[test]
fn first_reads_past_a_nullable_symbol() {
    use crate::parser::cfg::structs::{ParserNode, ParserProduction};
    static RULES: &[ParserProduction] = &[
        ParserProduction {
            left: NTerm::E,
            right: &[ParserNode::NTerm(NTerm::T), ParserNode::Term(ParserTokenKind::Eof)],
        },
        ParserProduction {
            left: NTerm::T,
            right: &[],
        },
        ParserProduction {
            left: NTerm::T,
            right: &[ParserNode::Term(ParserTokenKind::Int(0))],
        },
    ];
    let firsts = FirstPairBuilder::firsts(RULES);
    assert_eq!(
        firsts.set[&NTerm::E].0,
        HashSet::from([ParserTokenKind::Int(0), ParserTokenKind::Eof])
    );
    assert_eq!(firsts.nullable, HashSet::from([NTerm::T]));
}
