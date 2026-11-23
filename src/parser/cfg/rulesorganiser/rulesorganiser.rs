use std::collections::HashMap;

use crate::parser::cfg::{
    rulesorganiser::firsts::{self, FirstDerivation, FirstDerivationSet},
    structs::{NTerm, ParserNode, ParserProduction},
};

pub struct RuleOrganiser {
    rules: &'static [ParserProduction],
    // indexes in ascending order: the rules come out in the grammar's order
    sorter_first: HashMap<ParserNode, Vec<usize>>,
    sorter_left: HashMap<NTerm, Vec<usize>>,
    firsts: FirstDerivationSet,
}

impl RuleOrganiser {
    pub fn new(rules: &'static [ParserProduction]) -> Self {
        let mut sorter_first: HashMap<ParserNode, Vec<usize>> = HashMap::new();
        let mut sorter_left: HashMap<NTerm, Vec<usize>> = HashMap::new();
        let firsts: FirstDerivationSet = firsts::FirstPairBuilder::firsts(rules);

        for (i, rule) in rules.iter().enumerate() {
            if let Some(tok) = rule.right.first() {
                sorter_first.entry(tok.clone()).or_default().push(i);
            }
            sorter_left.entry(rule.left).or_default().push(i);
        }

        Self {
            rules,
            sorter_first,
            sorter_left,
            firsts,
        }
    }

    pub fn get(&self, ndx: usize) -> Option<&ParserProduction> {
        self.rules.get(ndx)
    }

    pub fn start_with_prod(&self, node: &ParserNode) -> impl Iterator<Item = &ParserProduction> {
        self.start_with(node).map(|&ndx| &self.rules[ndx])
    }

    pub fn start_with(&self, node: &ParserNode) -> impl Iterator<Item = &usize> {
        self.sorter_first.get(node).into_iter().flatten()
    }

    pub fn left_is(&self, nterm: &NTerm) -> impl Iterator<Item = &usize> {
        self.sorter_left.get(nterm).into_iter().flatten()
    }

    pub fn first(&self, node: &ParserNode) -> (FirstDerivation, bool) {
        self.firsts.first_of(node)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParserProduction> {
        self.rules.iter()
    }
}
