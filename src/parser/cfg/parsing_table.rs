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
    kernel_len: usize,
    pub(super) items: Vec<Item>,
    // sorted items reference all the nodes whose rule.right[dot] is the same
    sorted_items: HashMap<ParserNode, Vec<usize>>,
    pub(super) nexts: HashMap<ParserNode, usize>,
}

impl AutomataState {
    pub fn from_items(items: Vec<Item>) -> Self {
        let kernel_len = items.len();

        Self {
            kernel_len,
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
            let item = &self.items[i];
            let Some(rule) = rules.get(item.rule) else {
                unreachable!();
            };
            if item.dot >= rule.right.len() {
                i += 1;
                continue;
            }
            self.sorted_items
                .entry(rule.right[item.dot].clone())
                .or_default()
                .push(i);
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

    pub fn nexts(&self) -> Vec<(ParserNode, Self)> {
        let mut nexts: Vec<(ParserNode, Self)> = vec![];

        for elem in self.sorted_items.iter() {
            let next = AutomataState::from_items(
                elem.1
                    .iter()
                    .map(|ndx| {
                        let mut nw = self.items[*ndx].clone();
                        nw.dot += 1;
                        nw
                    })
                    .collect(),
            );
            nexts.push((elem.0.clone(), next));
        }

        return nexts;
    }

    fn same_kernel(&self, kernel: &[Item]) -> bool {
        let mine = &self.items[..self.kernel_len];
        mine.len() == kernel.len() && kernel.iter().all(|item| mine.contains(item))
    }
}

/// Only the kernels count, in any order: the same kernel always has the same closure
impl PartialEq for AutomataState {
    fn eq(&self, other: &Self) -> bool {
        self.same_kernel(&other.items[..other.kernel_len])
    }
}

#[derive(Default)]
pub struct Automata {
    states: Vec<AutomataState>,
}

impl Automata {
    fn automata_formation_loop(&mut self, rules: &RuleOrganiser) {
        let mut i = 0;
        while i < self.states.len() {
            let nexts = self.states[i].nexts();
            for (transi, mut state) in nexts {
                let ndx = match self.states.iter().position(|known| *known == state) {
                    Some(ndx) => ndx,
                    None => {
                        state.derives(rules);
                        self.states.push(state);
                        self.states.len() - 1
                    }
                };
                self.states[i].nexts.insert(transi, ndx);
            }
            i += 1;
        }
    }

    pub fn new(rules: &RuleOrganiser, start_prod: usize) -> Self {
        let mut first_state =
            AutomataState::from_items(vec![Item::new(start_prod, ParserTokenKind::Eof)]);
        first_state.derives(rules);
        println!("{}", first_state.display(rules));

        let mut me = Self {
            states: vec![first_state],
        };
        me.automata_formation_loop(rules);
        me
    }
}
