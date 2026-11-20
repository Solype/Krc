use super::structs::ParserProduction;

pub struct Parser {
    rules: &'static [ParserProduction],
}

impl Parser {
    pub fn new(rules: &'static [ParserProduction]) -> Self {
        Parser { rules }
    }
}
