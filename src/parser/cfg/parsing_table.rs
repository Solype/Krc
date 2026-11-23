use std::collections::HashMap;

use crate::parser::{
    cfg::{
        rulesorganiser::rulesorganiser::RuleOrganiser,
        structs::{ParserNode, ParserProduction},
    },
    parser_token::ParserTokenKind,
};

enum Action {}

#[derive(PartialEq, Eq, Clone)]
pub(super) struct Item {
    pub(super) rule: usize,
    pub(super) dot: usize,
    pub(super) lookahead: ParserTokenKind,
}

impl Item {
    pub fn new(rule: usize, lookahead: ParserTokenKind) -> Self {
        Self {
            rule,
            dot: 0,
            lookahead,
        }
    }
}

#[derive(Default)]
pub(super) struct AutomataState {
    pub(super) items: Vec<Item>,
    sorted_items: HashMap<ParserNode, usize>,
    pub(super) nexts: HashMap<ParserNode, usize>,
}

impl AutomataState {
    pub fn from_items(items: Vec<Item>) -> Self {
        Self {
            items,
            sorted_items: HashMap::new(),
            nexts: HashMap::new(),
        }
    }

    fn add_item(&mut self, new_item: Item) -> bool {
        if self.items.contains(&new_item) {
            return false;
        }
        self.items.push(new_item);
        return true;
    }

    /// FIRST(β a) for `[A → α • B β, a]`: the same for every rule of B, so computed once
    fn lookaheads(
        rules: &RuleOrganiser,
        rule: &ParserProduction,
        item: &Item,
    ) -> Vec<ParserTokenKind> {
        let mut lookaheads = vec![];
        for node in &rule.right[item.dot + 1..] {
            let (firsts, nullable) = rules.first(node);
            lookaheads.extend(firsts.iter().cloned());
            if !nullable {
                return lookaheads;
            }
        }
        lookaheads.push(item.lookahead.clone());
        lookaheads
    }

    pub fn derives(&mut self, rules: &RuleOrganiser) {
        let mut i = 0;

        while i < self.items.len() {
            let item = self.items[i].clone();
            let Some(rule) = rules.get(item.rule) else {
                unreachable!();
            };
            if item.dot >= rule.right.len() {
                i += 1;
                continue;
            }
            let ParserNode::NTerm(nterm) = &rule.right[item.dot] else {
                i += 1;
                continue;
            };
            let lookaheads = Self::lookaheads(rules, rule, &item);
            for rule_ndx in rules.left_is(nterm) {
                for lookahead in &lookaheads {
                    self.add_item(Item::new(*rule_ndx, lookahead.clone()));
                }
            }
            i += 1;
        }
    }
}

#[derive(Default)]
pub struct Automata {
    states: Vec<AutomataState>,
    map: HashMap<(usize, ParserNode), Action>,
}

impl Automata {
    pub fn new(rules: &RuleOrganiser, start_prod: usize) -> Self {
        let mut first_state =
            AutomataState::from_items(vec![Item::new(start_prod, ParserTokenKind::Eof)]);
        first_state.derives(rules);
        println!("{}", first_state.display(rules));
        Self::default()
    }
}
