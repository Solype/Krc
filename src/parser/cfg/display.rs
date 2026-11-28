use std::collections::{HashMap, HashSet};
use std::fmt;

use super::parser::TreeNode;
use super::parsing_table::automata_state::AutomataState;
use super::parsing_table::item::Item;
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

/// A node only holds its rule's index: `tree.display(&rules)` brings the rules to print it
pub struct TreeNodeDisplay<'a> {
    tree: &'a TreeNode,
    rules: &'a RuleOrganiser,
}

impl TreeNode {
    pub fn display<'a>(&'a self, rules: &'a RuleOrganiser) -> TreeNodeDisplay<'a> {
        TreeNodeDisplay { tree: self, rules }
    }
}

/// One line per node, its children below it with guides: `Statement [12]` (the rule's index, what
/// the post-parser matches on), a leaf as its token with its value, `Name("compte")`
impl fmt::Display for TreeNodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.lines(f, self.tree, "", "")
    }
}

impl TreeNodeDisplay<'_> {
    /// `head` goes before this node's label, `prefix` before the lines of its children
    fn lines(&self, f: &mut fmt::Formatter, node: &TreeNode, head: &str, prefix: &str) -> fmt::Result {
        match node {
            TreeNode::Leaf(token) => writeln!(f, "{head}{:?}", token.kind),
            TreeNode::Node { rule, children } => {
                match self.rules.get(*rule) {
                    Some(production) => writeln!(f, "{head}{:?} [{rule}]", production.left)?,
                    None => writeln!(f, "{head}rule {rule} unknown")?,
                }
                for (i, child) in children.iter().enumerate() {
                    let last = i + 1 == children.len();
                    let head = format!("{prefix}{}", if last { "└── " } else { "├── " });
                    let next = format!("{prefix}{}", if last { "    " } else { "│   " });
                    self.lines(f, child, &head, &next)?;
                }
                Ok(())
            }
        }
    }
}

pub(super) fn node_name(node: &ParserNode) -> String {
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
