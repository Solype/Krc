use crate::parser::{
    cfg::{
        parsing_table::parsing_table::{Action, ParsingTable},
        rulesorganiser::rulesorganiser::RuleOrganiser,
        structs::ParserNode,
    },
    parser_token::ParserToken,
};

/// The grammar starts with its start rule
const START_RULE: usize = 0;

pub struct Parser {
    rules: RuleOrganiser,
    table: ParsingTable,
}

#[derive(Debug, PartialEq)]
pub enum TreeNode {
    Leaf(ParserToken),
    /// `rule` is an index of the grammar; a nullable one can have no children
    Node {
        rule: usize,
        children: Vec<TreeNode>,
    },
}

impl Parser {
    pub fn new(rules: RuleOrganiser) -> Result<Self, String> {
        let table = ParsingTable::new(&rules, START_RULE)?;
        Ok(Parser { rules, table })
    }

    pub fn run(&self, tokens: &[ParserToken]) -> Result<TreeNode, String> {
        let mut stack: Vec<TreeNode> = vec![];
        let mut pos: usize = 0;
        let mut stack_state: Vec<usize> = vec![0];

        loop {
            let Some(token) = tokens.get(pos) else {
                return Err("erreur".to_string());
            };
            let tok = ParserNode::Term(token.kind.clone());
            match self.table.map.get(&(*stack_state.last().unwrap(), tok)) {
                // the Eof of the start rule is never shifted: it is not in the tree
                Some(Action::Accept) => {
                    return Ok(TreeNode::Node {
                        rule: START_RULE,
                        children: stack,
                    });
                }
                Some(Action::Reduce(rulendx)) => {
                    let Some(r) = self.rules.get(*rulendx) else {
                        return Err("erreur".to_string());
                    };
                    let children = stack.split_off(stack.len() - r.right.len());
                    stack_state.truncate(stack_state.len() - r.right.len());
                    let Some(Action::Goto(newstate)) = self
                        .table
                        .map
                        .get(&(*stack_state.last().unwrap(), ParserNode::NTerm(r.left)))
                    else {
                        unreachable!();
                    };
                    stack.push(TreeNode::Node {
                        rule: *rulendx,
                        children,
                    });
                    stack_state.push(*newstate);
                }
                Some(Action::Shift(newstate)) => {
                    stack_state.push(*newstate);
                    stack.push(TreeNode::Leaf(token.clone()));
                    pos += 1;
                }
                Some(Action::Goto(_)) => unreachable!(),
                None => return Err("erreur".to_string()),
            }
        }
    }
}
