use crate::parser::cfg::{
    parsing_table::automata::Automata, rulesorganiser::rulesorganiser::RuleOrganiser,
};

pub struct Parser {
    rules: RuleOrganiser,
}

impl Parser {
    pub fn new(rules: RuleOrganiser) -> Self {
        let _automata = Automata::new(&rules, 0);
        Parser { rules }
    }
}
