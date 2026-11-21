use crate::parser::cfg::{firsts, follows};

use super::structs::ParserProduction;

pub struct Parser {
    rules: &'static [ParserProduction],
}

impl Parser {
    pub fn new(rules: &'static [ParserProduction]) -> Self {
        let firsts = firsts::FirstPairBuilder::firsts(rules);
        let follows =
            follows::FollowsDerivationBuilder::follows(rules, &firsts, super::structs::NTerm::E);
        println!("FIRST:\n{firsts}");
        println!("FOLLOW:\n{follows}");

        Parser { rules }
    }
}
