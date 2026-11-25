use crate::parser::{
    cfg::{
        parsing_table::parsing_table::{Action, ParsingTable},
        rulesorganiser::rulesorganiser::RuleOrganiser,
        structs::ParserNode,
    },
    parser_token::ParserToken,
};

pub struct Parser {
    rules: RuleOrganiser,
    table: ParsingTable,
}

pub struct TreeNode {
    node: ParserNode,
    children: Vec<TreeNode>,
}

impl Parser {
    pub fn new(rules: RuleOrganiser) -> Result<Self, String> {
        let table = ParsingTable::new(&rules, 0)?;
        Ok(Parser { rules, table })
    }

    pub fn run(&self, tokens: &[ParserToken]) -> Result<(), String> {
        let mut stack: Vec<ParserNode> = vec![];
        let mut pos: usize = 0;
        let mut stack_state: Vec<usize> = vec![0];

        loop {
            let Some(token) = tokens.get(pos) else {
                return Err("erreur".to_string());
            };
            let tok = ParserNode::Term(token.kind.clone());
            match self
                .table
                .map
                .get(&(*stack_state.last().unwrap(), tok.clone()))
            {
                Some(Action::Accept) => return Ok(()),
                Some(Action::Reduce(rulendx)) => {
                    let Some(r) = self.rules.get(*rulendx) else {
                        return Err("erreur".to_string());
                    };
                    stack.truncate(stack.len() - r.right.len());
                    stack_state.truncate(stack_state.len() - r.right.len());
                    let left = ParserNode::NTerm(r.left);
                    let Some(Action::Goto(newstate)) = self
                        .table
                        .map
                        .get(&(*stack_state.last().unwrap(), left.clone()))
                    else {
                        unreachable!();
                    };
                    stack.push(left);
                    stack_state.push(*newstate);
                }
                Some(Action::Shift(newstate)) => {
                    stack_state.push(*newstate);
                    stack.push(tok);
                    pos += 1;
                }
                Some(Action::Goto(_)) => unreachable!(),
                None => return Err("erreur".to_string()),
            }
        }
    }
}
