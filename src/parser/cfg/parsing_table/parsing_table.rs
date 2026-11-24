use std::collections::HashMap;

use crate::parser::{
    cfg::{
        display::node_name,
        parsing_table::{
            automata::Automata,
            automata_state::AutomataState,
            parsing_table::Action::{Accept, Goto, Reduce, Shift},
        },
        rulesorganiser::rulesorganiser::RuleOrganiser,
        structs::ParserNode,
    },
    parser_token::ParserTokenKind,
};

pub enum Action {
    Shift(usize),
    Goto(usize),
    Reduce(usize),
    Accept,
}

#[derive(Default)]
pub struct ParsingTable {
    pub map: HashMap<(usize, ParserNode), Action>,
}

impl ParsingTable {
    fn insert(&mut self, state_id: usize, node: ParserNode, action: Action) -> Result<(), String> {
        let name = node_name(&node);
        if self.map.insert((state_id, node), action).is_some() {
            return Err(format!("conflict in state {state_id} on {name}"));
        }
        Ok(())
    }

    fn fill_table_from_state(
        &mut self,
        state_id: usize,
        state: &AutomataState,
        rules: &RuleOrganiser,
        start_prod: usize,
    ) -> Result<(), String> {
        for (node, next_state) in state.nexts.iter() {
            let action = match node {
                ParserNode::NTerm(_) => Goto(*next_state),
                // the lexer sends a single Eof: nothing is left to reduce the start rule on
                ParserNode::Term(ParserTokenKind::Eof) => Accept,
                ParserNode::Term(_) => Shift(*next_state),
            };
            self.insert(state_id, node.clone(), action)?;
        }
        for item in state.items.iter() {
            let Some(rule) = rules.get(item.rule) else {
                unreachable!();
            };
            if item.dot < rule.right.len() {
                continue;
            }
            // shortcut: wrong if the start symbol has several rules or can end one of its own
            // rules, add a start rule `S' → S` to such a grammar
            let action = if item.rule == start_prod && item.lookahead == ParserTokenKind::Eof {
                Accept
            } else {
                Reduce(item.rule)
            };
            self.insert(state_id, ParserNode::Term(item.lookahead.clone()), action)?;
        }
        Ok(())
    }

    pub fn new(rules: &RuleOrganiser, start_prod: usize) -> Result<Self, String> {
        let mut me = Self::default();

        let automata = Automata::new(rules, start_prod);

        for (i, state) in automata.states.iter().enumerate() {
            me.fill_table_from_state(i, state, rules, start_prod)?;
        }

        Ok(me)
    }
}
