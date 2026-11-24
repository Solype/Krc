use std::collections::HashMap;

use crate::parser::cfg::structs::ParserNode;

enum Action {}

#[derive(Default)]
pub struct ParsingTable {
    map: HashMap<(usize, ParserNode), Action>,
}
