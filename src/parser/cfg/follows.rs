/// resource : https://www.youtube.com/watch?v=oOCromcWnfc
use std::collections::{HashMap, HashSet};

use crate::parser::{
    cfg::{
        firsts::FirstDerivationSet,
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
    fn process_follows_from_1_rule(
        &mut self,
        firsts: &FirstDerivationSet,
        rule: &ParserProduction,
    ) {
        for (i, node) in rule.right.iter().enumerate() {
            let ParserNode::NTerm(nterm) = node else {
                continue;
            };
            match rule.right.get(i + 1) {
                // A → … B : FOLLOW(B) gets FOLLOW(A)
                None => {
                    self.eq_statements.insert((*nterm, rule.left));
                }
                // B a : FOLLOW(B) gets a
                Some(ParserNode::Term(term)) => {
                    self.follow_derivatives
                        .entry(*nterm)
                        .or_default()
                        .0
                        .insert(term.clone());
                }
                // B C : FOLLOW(B) gets FIRST(C)
                Some(ParserNode::NTerm(next)) => {
                    let Some(firsts_of_next) = firsts.0.get(next) else {
                        unimplemented!();
                    };
                    self.follow_derivatives
                        .entry(*nterm)
                        .or_default()
                        .0
                        .extend(firsts_of_next.0.clone());
                }
            }
        }
    }

    fn process_follows_derivative(
        &mut self,
        rules: &'static [ParserProduction],
        firsts: &FirstDerivationSet,
    ) {
        for rule in rules {
            self.process_follows_from_1_rule(firsts, rule);
        }
    }

    pub fn follows(
        rules: &'static [ParserProduction],
        firsts: &FirstDerivationSet,
        start: NTerm,
    ) -> FollowDerivationSet {
        let mut me = Self::default();

        me.follow_derivatives
            .entry(start)
            .or_default()
            .0
            .insert(ParserTokenKind::Eof);

        me.process_follows_derivative(rules, firsts);
        FollowDerivationSet(me.follow_derivatives)
    }
}
