use super::automata_state::AutomataState;
use super::item::Item;
use crate::parser::{
    cfg::rulesorganiser::rulesorganiser::RuleOrganiser, parser_token::ParserTokenKind,
};

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
