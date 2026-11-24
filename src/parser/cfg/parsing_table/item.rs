use crate::parser::parser_token::ParserTokenKind;

#[derive(PartialEq, Eq, Clone)]
pub struct Item {
    pub rule: usize,
    pub dot: usize,
    pub lookahead: ParserTokenKind,
}

impl Item {
    pub fn new(rule: usize, lookahead: ParserTokenKind) -> Self {
        Self {
            rule,
            dot: 0,
            lookahead,
        }
    }
}
