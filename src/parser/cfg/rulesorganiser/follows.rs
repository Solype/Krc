/// resource : https://www.youtube.com/watch?v=oOCromcWnfc
use std::collections::{HashMap, HashSet};

use crate::parser::{
    cfg::{
        rulesorganiser::rulesorganiser::RuleOrganiser,
        structs::{NTerm, ParserNode, ParserProduction},
    },
    parser_token::ParserTokenKind,
};

#[derive(Debug, Default)]
pub struct FollowDerivation(pub HashSet<ParserTokenKind>);

#[derive(Debug, Default)]
pub struct FollowDerivationSet(pub HashMap<NTerm, FollowDerivation>);

#[derive(Default, Debug)]
pub struct FollowsDerivationBuilder {
    pub eq_statements: HashSet<(NTerm, NTerm)>,
    pub follow_derivatives: HashMap<NTerm, FollowDerivation>,
}

impl FollowsDerivationBuilder {
    fn process_follows_for_1_nterm(
        &mut self,
        i: usize,
        nterm: &NTerm,
        rules: &RuleOrganiser,
        rule: &ParserProduction,
    ) {
        let mut j = 1;
        let mut run = false;

        loop {
            match rule.right.get(i + j) {
                // A → … B : FOLLOW(B) gets FOLLOW(A)
                None => {
                    self.eq_statements.insert((*nterm, rule.left));
                }
                // B X : FOLLOW(B) gets FIRST(X), FIRST(a) = { a }; X nullable: the next symbol too
                Some(next) => {
                    let (first, nullable) = rules.first(next);
                    run = nullable;
                    self.follow_derivatives
                        .entry(*nterm)
                        .or_default()
                        .0
                        .extend(first.0);
                }
            }
            if !run {
                break;
            } else {
                j += 1;
                run = false;
            }
        }
    }

    fn process_follows_derivative(&mut self, rules: &RuleOrganiser) {
        for rule in rules.iter() {
            for (i, node) in rule.right.iter().enumerate() {
                let ParserNode::NTerm(nterm) = node else {
                    continue;
                };
                self.process_follows_for_1_nterm(i, nterm, rules, rule);
            }
        }
    }

    fn resolve_eq_statement(&mut self) {
        let mut changed = true;

        while changed {
            changed = false;
            for (left, right) in self.eq_statements.iter() {
                // copied first: the map cannot be borrowed twice at once
                let source = self
                    .follow_derivatives
                    .get(right)
                    .map(|first| first.0.clone())
                    .unwrap_or_default();
                let target = &mut &mut self.follow_derivatives.entry(*left).or_default().0;
                let before = target.len();
                target.extend(source);
                changed |= target.len() != before;
            }
        }
    }

    pub fn follows(rules: &RuleOrganiser, start: NTerm) -> FollowDerivationSet {
        let mut me = Self::default();

        me.follow_derivatives
            .entry(start)
            .or_default()
            .0
            .insert(ParserTokenKind::Eof);

        me.process_follows_derivative(rules);
        me.resolve_eq_statement();
        FollowDerivationSet(me.follow_derivatives)
    }
}
