use crate::parser::cfg::parsing_table::automata_state::AutomataState;
use crate::parser::cfg::parsing_table::item::Item;
use crate::parser::cfg::rules::TEST_RULES;
use crate::parser::cfg::rulesorganiser::rulesorganiser::RuleOrganiser;
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
