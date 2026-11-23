use std::collections::{HashMap, HashSet};
use std::fmt;

use super::parsing_table::{AutomataState, Item};
use super::rulesorganiser::firsts::{FirstDerivation, FirstDerivationSet};
use super::rulesorganiser::follows::{FollowDerivation, FollowDerivationSet};
use super::rulesorganiser::rulesorganiser::RuleOrganiser;
use super::structs::{NTerm, ParserNode};
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

/// One line per non-terminal, in the order of `NTerm`: `E: {Name, Punctuation(LParen)}`, then
/// `nullable: {EPrime, TPrime}`
impl fmt::Display for FirstDerivationSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lines(f, &self.set)?;
        let mut nullable: Vec<&NTerm> = self.nullable.iter().collect();
        nullable.sort_by_key(|nterm| **nterm as usize);
        let names: Vec<String> = nullable.iter().map(|nterm| format!("{nterm:?}")).collect();
        writeln!(f, "nullable: {{{}}}", names.join(", "))
    }
}

impl fmt::Display for FollowDerivationSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        lines(f, &self.0)
    }
}

/// An item only holds its rule's index: `state.display(&rules)` brings the rules to print it
pub(super) struct AutomataStateDisplay<'a> {
    state: &'a AutomataState,
    rules: &'a RuleOrganiser,
}

impl AutomataState {
    pub(super) fn display<'a>(&'a self, rules: &'a RuleOrganiser) -> AutomataStateDisplay<'a> {
        AutomataStateDisplay { state: self, rules }
    }
}

/// One line per item in insertion order, `[E → T • EPrime, Eof]`, then one line per transition,
/// sorted: `T → 3`
impl fmt::Display for AutomataStateDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for item in &self.state.items {
            writeln!(f, "{}", self.item(item))?;
        }
        let mut nexts: Vec<(String, usize)> = self
            .state
            .nexts
            .iter()
            .map(|(node, state)| (node_name(node), *state))
            .collect();
        nexts.sort();
        for (node, state) in nexts {
            writeln!(f, "{node} → {state}")?;
        }
        Ok(())
    }
}

impl AutomataStateDisplay<'_> {
    fn item(&self, item: &Item) -> String {
        let Some(rule) = self.rules.get(item.rule) else {
            return format!(
                "[rule {} unknown, {}]",
                item.rule,
                terminal(&item.lookahead)
            );
        };
        let mut right: Vec<String> = rule.right.iter().map(node_name).collect();
        right.insert(item.dot.min(right.len()), "•".to_string());
        format!(
            "[{:?} → {}, {}]",
            rule.left,
            right.join(" "),
            terminal(&item.lookahead)
        )
    }
}

fn node_name(node: &ParserNode) -> String {
    match node {
        ParserNode::Term(kind) => terminal(kind),
        ParserNode::NTerm(nterm) => format!("{nterm:?}"),
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
