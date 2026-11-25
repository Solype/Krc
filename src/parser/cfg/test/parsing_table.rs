use crate::lexer::lexer_token::{Operator, Punctuation};
use crate::lexer::span::Span;
use crate::parser::cfg::parser::{Parser, TreeNode};
use crate::parser::cfg::parsing_table::automata_state::AutomataState;
use crate::parser::cfg::parsing_table::item::Item;
use crate::parser::cfg::parsing_table::parsing_table::{Action, ParsingTable};
use crate::parser::cfg::rules::{RULES, TEST_RULES};
use crate::parser::cfg::rulesorganiser::rulesorganiser::RuleOrganiser;
use crate::parser::parser_token::{ParserToken, ParserTokenKind};

#[test]
fn states_are_equal_when_their_kernels_are() {
    let first = Item::new(0, ParserTokenKind::Eof);
    let second = Item::new(3, ParserTokenKind::Eof);
    let state = AutomataState::from_items(vec![first.clone(), second.clone()]);
    let mut closed = AutomataState::from_items(vec![second, first.clone()]);
    closed.derives(&RuleOrganiser::new(TEST_RULES));

    assert!(state == closed);
    assert!(state != AutomataState::from_items(vec![first]));
}

/// Parses `tokens` (then Eof); the span of a token is its index
fn parse(tokens: &[ParserTokenKind]) -> Result<TreeNode, String> {
    let tokens: Vec<ParserToken> = tokens
        .iter()
        .cloned()
        .chain([ParserTokenKind::Eof])
        .enumerate()
        .map(|(i, kind)| ParserToken::new(kind, Span::new(i, i + 1)))
        .collect();
    Parser::new(RuleOrganiser::new(TEST_RULES)).unwrap().run(&tokens)
}

fn accepts(tokens: &[ParserTokenKind]) -> bool {
    parse(tokens).is_ok()
}

#[test]
fn the_tree_keeps_the_rules_and_the_real_tokens() {
    let x = ParserTokenKind::Name("x".to_string());
    let y = ParserTokenKind::Name("y".to_string());
    let plus = ParserTokenKind::Operator(Operator::Plus);
    let node = |rule, children| TreeNode::Node { rule, children };
    let leaf = |kind: &ParserTokenKind, i| {
        TreeNode::Leaf(ParserToken::new(kind.clone(), Span::new(i, i + 1)))
    };
    // T → F TPrime, F → Name, TPrime → ε
    let term = |kind, i| node(3, vec![node(6, vec![leaf(kind, i)]), node(5, vec![])]);

    assert_eq!(
        parse(&[x.clone(), plus.clone(), y.clone()]).unwrap(),
        node(
            0,
            vec![
                term(&x, 0),
                node(1, vec![leaf(&plus, 1), term(&y, 2), node(2, vec![])]),
            ]
        )
    );
}

#[test]
fn the_parser_accepts_the_expressions_and_nothing_else() {
    let name = || ParserTokenKind::Name("x".to_string());
    let plus = || ParserTokenKind::Operator(Operator::Plus);
    let star = || ParserTokenKind::Operator(Operator::Star);
    let open = || ParserTokenKind::Punctuation(Punctuation::LParen);
    let close = || ParserTokenKind::Punctuation(Punctuation::RParen);

    assert!(accepts(&[name()]));
    assert!(accepts(&[name(), plus(), name(), star(), name()]));
    assert!(accepts(&[
        open(), open(), name(), plus(), name(), close(), close(), star(), name()
    ]));
    assert!(!accepts(&[]));
    assert!(!accepts(&[name(), plus()]));
    assert!(!accepts(&[open(), name()]));
    assert!(!accepts(&[name(), close()]));
    assert!(!accepts(&[name(), name()]));
}

#[test]
fn the_grammar_has_no_conflict() {
    let table = ParsingTable::new(&RuleOrganiser::new(RULES), 0).unwrap();
    let accepts = table.map.values().filter(|action| matches!(action, Action::Accept));
    // before Eof, and the state after it, never reached
    assert_eq!(accepts.count(), 2);
}
