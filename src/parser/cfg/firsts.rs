use std::collections::{HashMap, HashSet};

use crate::parser::{
    cfg::structs::{ParserNode, ParserProduction},
    parser_token::ParserTokenKind,
};

/// resource : https://www.youtube.com/watch?v=oOCromcWnfc
///
use super::structs::NTerm;

#[derive(Debug, Default)]
pub struct FirstDerivation(pub HashSet<ParserTokenKind>);

#[derive(Debug, Default)]
pub struct FirstDerivationSet {
    pub set: HashMap<NTerm, FirstDerivation>,
    pub nullable: HashSet<NTerm>,
}

#[derive(Debug, Default)]
pub struct FirstPairBuilder {
    pub eq_statements: HashSet<(NTerm, NTerm)>,
    pub first_derivatives: HashMap<NTerm, FirstDerivation>,
    pub nullable: HashSet<NTerm>,
}

impl FirstPairBuilder {
    /// For each `(left, right)`, FIRST(left) gets FIRST(right). Repeated until no set grows, so
    /// chains (E → T → F) and loops (A → B, B → A) both end
    fn resolve_eq_statements(&mut self) {
        let mut changed = true;

        while changed {
            changed = false;
            for (left, right) in self.eq_statements.iter() {
                if self.nullable.contains(right) {
                    self.nullable.insert(*left);
                }

                // copied first: the map cannot be borrowed twice at once
                let source = self
                    .first_derivatives
                    .get(right)
                    .map(|first| first.0.clone())
                    .unwrap_or_default();
                let target = &mut self.first_derivatives.entry(*left).or_default().0;
                let before = target.len();
                target.extend(source);
                changed |= target.len() != before;
            }
        }
    }

    fn process_first_derivative(&mut self, rules: &'static [ParserProduction]) {
        for rule in rules {
            if rule.right.len() == 0 {
                self.nullable.insert(rule.left);
                continue;
            }
            match &rule.right[0] {
                ParserNode::Term(term) => self
                    .first_derivatives
                    .entry(rule.left)
                    .or_default()
                    .0
                    .insert(term.clone()),
                ParserNode::NTerm(nterm) => self.eq_statements.insert((rule.left, *nterm)),
            };
        }
    }

    pub fn firsts(rules: &'static [ParserProduction]) -> FirstDerivationSet {
        let mut me = Self::default();

        me.process_first_derivative(rules);
        me.resolve_eq_statements();
        return FirstDerivationSet {
            set: me.first_derivatives,
            nullable: me.nullable,
        };
    }
}
