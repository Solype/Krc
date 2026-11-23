use std::collections::{HashMap, HashSet};

use crate::parser::{
    cfg::structs::{ParserNode, ParserProduction},
    parser_token::ParserTokenKind,
};

/// resource : https://www.youtube.com/watch?v=oOCromcWnfc
///
use crate::parser::cfg::structs::NTerm;

#[derive(Debug, Default, Clone)]
pub struct FirstDerivation(pub HashSet<ParserTokenKind>);

impl std::ops::Deref for FirstDerivation {
    type Target = HashSet<ParserTokenKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct FirstDerivationSet {
    pub set: HashMap<NTerm, FirstDerivation>,
    pub nullable: HashSet<NTerm>,
}

impl FirstDerivationSet {
    pub fn first_of(&self, node: &ParserNode) -> (FirstDerivation, bool) {
        match node {
            ParserNode::NTerm(nterm) => (
                self.set.get(nterm).cloned().unwrap_or_default(),
                self.nullable.contains(nterm),
            ),
            ParserNode::Term(tok) => (FirstDerivation(HashSet::from([tok.clone()])), false),
        }
    }
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
                continue;
            }
            let mut is_nullable = true;
            let mut i = 0;
            while is_nullable {
                is_nullable = false;
                match &rule.right.get(i) {
                    None => self.nullable.insert(rule.left),
                    Some(ParserNode::Term(term)) => self
                        .first_derivatives
                        .entry(rule.left)
                        .or_default()
                        .0
                        .insert(term.clone()),
                    Some(ParserNode::NTerm(nterm)) => {
                        is_nullable = self.nullable.contains(nterm);
                        self.eq_statements.insert((rule.left, *nterm))
                    }
                };
                i += 1;
            }
        }
    }

    pub fn get_nullable(&mut self, rules: &'static [ParserProduction]) {
        // a rule with a terminal never derives ε; an empty rule (ε) is kept: `all` is true on it
        let potentially_nullable: Vec<&ParserProduction> = rules
            .iter()
            .filter(|rule| {
                rule.right
                    .iter()
                    .all(|node| matches!(node, ParserNode::NTerm(_)))
            })
            .collect();

        // A → B C is nullable once B and C are: repeated until no new nullable
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &potentially_nullable {
                let all_nullable = rule.right.iter().all(|node| {
                    matches!(node, ParserNode::NTerm(nterm) if self.nullable.contains(nterm))
                });
                if all_nullable {
                    changed |= self.nullable.insert(rule.left);
                }
            }
        }
    }

    pub fn firsts(rules: &'static [ParserProduction]) -> FirstDerivationSet {
        let mut me = Self::default();

        me.get_nullable(rules);
        me.process_first_derivative(rules);
        me.resolve_eq_statements();
        return FirstDerivationSet {
            set: me.first_derivatives,
            nullable: me.nullable,
        };
    }
}
