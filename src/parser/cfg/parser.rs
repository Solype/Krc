use crate::parser::cfg::{
    parsing_table::parsing_table::ParsingTable, rulesorganiser::rulesorganiser::RuleOrganiser,
    structs::ParserNode,
};

pub struct Parser {
    rules: RuleOrganiser,
    table: ParsingTable,
}

impl Parser {
    pub fn new(rules: RuleOrganiser) -> Result<Self, String> {
        let table = ParsingTable::new(&rules, 0)?;
        Ok(Parser { rules, table })
    }

    pub fn run(tokens: &Vec<ParserNode>) {}
}
