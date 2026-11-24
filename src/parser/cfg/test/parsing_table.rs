use crate::lexer::lexer_token::{Operator, Punctuation};
use crate::parser::cfg::parsing_table::automata_state::AutomataState;
use crate::parser::cfg::parsing_table::item::Item;
use crate::parser::cfg::parsing_table::parsing_table::{Action, ParsingTable};
use crate::parser::cfg::rules::{RULES, TEST_RULES};
use crate::parser::cfg::rulesorganiser::rulesorganiser::RuleOrganiser;
use crate::parser::cfg::structs::ParserNode;
use crate::parser::parser_token::ParserTokenKind;

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

/// Runs the table on `tokens` (then Eof), true when it accepts
fn accepts(tokens: &[ParserTokenKind]) -> bool {
    let rules = RuleOrganiser::new(TEST_RULES);
    let table = ParsingTable::new(&rules, 0).unwrap();
    let mut tokens = tokens.iter().cloned().chain([ParserTokenKind::Eof]).peekable();
    let mut stack = vec![0];
    loop {
        let token = ParserNode::Term(tokens.peek().unwrap().clone());
        match table.map.get(&(*stack.last().unwrap(), token)) {
            Some(Action::Shift(state)) => {
                tokens.next();
                stack.push(*state);
            }
            Some(Action::Reduce(rule)) => {
                let rule = rules.get(*rule).unwrap();
                stack.truncate(stack.len() - rule.right.len());
                let left = ParserNode::NTerm(rule.left);
                let Some(Action::Goto(state)) = table.map.get(&(*stack.last().unwrap(), left))
                else {
                    panic!("no goto");
                };
                stack.push(*state);
            }
            Some(Action::Accept) => return true,
            Some(Action::Goto(_)) => unreachable!(),
            None => return false,
        }
    }
}

#[test]
fn the_table_accepts_the_expressions_and_nothing_else() {
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
