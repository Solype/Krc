use std::collections::{HashMap, HashSet};
use std::fmt;

use super::firsts::{FirstDerivation, FirstDerivationSet};
use super::follows::{FollowDerivation, FollowDerivationSet};
use super::structs::NTerm;
use crate::parser::parser_token::ParserTokenKind;

/// `{Name, Punctuation(LParen)}`, sorted: a `HashSet` has no stable order
impl fmt::Display for FirstDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        terminals(f, &self.0)
    }
}

impl fmt::Display for FollowDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        terminals(f, &self.0)
    }
}

/// One line per non-terminal, in the order of `NTerm`: `E: {Name, Punctuation(LParen)}`
impl fmt::Display for FirstDerivationSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lines(f, &self.0)
    }
}

impl fmt::Display for FollowDerivationSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lines(f, &self.0)
    }
}

fn terminals(f: &mut fmt::Formatter, set: &HashSet<ParserTokenKind>) -> fmt::Result {
    let mut names: Vec<String> = set.iter().map(terminal).collect();
    names.sort();
    write!(f, "{{{}}}", names.join(", "))
}

fn lines<T: fmt::Display>(f: &mut fmt::Formatter, map: &HashMap<NTerm, T>) -> fmt::Result {
    let mut entries: Vec<(&NTerm, &T)> = map.iter().collect();
    entries.sort_by_key(|(nterm, _)| **nterm as usize);
    for (nterm, set) in entries {
        writeln!(f, "{nterm:?}: {set}")?;
    }
    Ok(())
}

/// A terminal of a rule carries a placeholder value (`Name("")`, `Int(0)`): only its kind is shown
fn terminal(kind: &ParserTokenKind) -> String {
    match kind {
        ParserTokenKind::Name(_) => "Name".to_string(),
        ParserTokenKind::Ident(_) => "Ident".to_string(),
        ParserTokenKind::Counter(_) => "Counter".to_string(),
        ParserTokenKind::Int(_) => "Int".to_string(),
        ParserTokenKind::Float(_) => "Float".to_string(),
        ParserTokenKind::Str(_) => "Str".to_string(),
        ParserTokenKind::Bool(_) => "Bool".to_string(),
        ParserTokenKind::Verb { ending, .. } => format!("Verb({ending:?})"),
        other => format!("{other:?}"),
    }
}
