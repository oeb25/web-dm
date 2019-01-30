#![feature(box_syntax, slice_patterns)]

use indexmap::{IndexMap, IndexSet};
use std::collections::HashSet;

mod ast;
mod parse;
pub mod tableau;

pub use crate::ast::Connective;
pub use crate::parse::{parse, ParseError};

impl Connective {
    pub fn all_variables(&self) -> Vec<String> {
        let mut set = IndexSet::new();
        self.all_variables_helper(&mut HashSet::new(), &mut set);

        set.iter().rev().cloned().collect()
    }

    fn all_variables_helper(&self, ignore: &mut HashSet<String>, set: &mut IndexSet<String>) {
        match self {
            Connective::Not(x) => x.all_variables_helper(ignore, set),
            Connective::ForAll(r, x) | Connective::Exists(r, x) => {
              let mut ignore = ignore.clone();
              ignore.insert(r.clone());
              x.all_variables_helper(&mut ignore, set)
            },
            Connective::Var(x) => {
              if !ignore.contains(x) {
                set.insert(x.to_string());
              }
            }
            Connective::Predicate(_, args) => {
                for arg in args {
                  if !ignore.contains(arg) {
                    set.insert(arg.to_string());
                  }
                }
            }
            Connective::And(a, b)
            | Connective::Or(a, b)
            | Connective::Implicate(a, b)
            | Connective::Biimplicate(a, b) => {
                a.all_variables_helper(ignore, set);
                b.all_variables_helper(ignore, set);
            }
        }
    }
    pub fn all_atomics(&self) -> Vec<Connective> {
        let mut set = IndexSet::new();
        self.all_atomics_helper(&mut HashSet::new(), &mut set);

        set.iter().rev().cloned().collect()
    }

    fn all_atomics_helper(&self, ignore: &mut HashSet<Connective>, set: &mut IndexSet<Connective>) {
        match self {
            Connective::Not(x) => x.all_atomics_helper(ignore, set),
            Connective::ForAll(r, x) | Connective::Exists(r, x) => {
              let mut ignore = ignore.clone();
              ignore.insert(Connective::Var(r.clone()));
              x.all_atomics_helper(&mut ignore, set)
            },
            Connective::ForAll(r, x) | Connective::Exists(r, x) => {},
            Connective::Var(_) | Connective::Predicate(_, _) => {
              if !ignore.contains(self) {
                set.insert(self.clone());
              }
            }
            Connective::And(a, b)
            | Connective::Or(a, b)
            | Connective::Implicate(a, b)
            | Connective::Biimplicate(a, b) => {
                a.all_atomics_helper(ignore, set);
                b.all_atomics_helper(ignore, set);
            }
        }
    }

    pub fn all_sub_connectives(&self, is_first: bool) -> Vec<Connective> {
        match self {
            Connective::Not(x) => {
                if is_first {
                    let mut x_s = x.all_sub_connectives(false);
                    x_s.push(self.clone());
                    x_s
                } else {
                    x.all_sub_connectives(false)
                }
            },
            Connective::Var(_) => vec![],
            Connective::Predicate(_, _) => vec![],
            Connective::And(a, b)
            | Connective::Or(a, b)
            | Connective::Implicate(a, b)
            | Connective::Biimplicate(a, b) => {
                let mut a_s = a.all_sub_connectives(false);
                let mut b_s = b.all_sub_connectives(false);
                a_s.append(&mut b_s);

                a_s.push(self.clone());

                a_s
            }
            Connective::ForAll(_, a) | Connective::Exists(_, a) => {
                let mut sub = a.all_sub_connectives(false);
                sub.push(self.clone());
                sub
            }
        }
    }

    pub fn solve(&self, variables: &IndexMap<String, bool>) -> bool {
        match self {
            Connective::Not(x) => !x.solve(variables),
            Connective::Var(x) => variables.get(x).cloned().unwrap_or(false), // todo
            Connective::Predicate(_, _) => false,                             // todo
            Connective::And(a, b) => a.solve(variables) && b.solve(variables),
            Connective::Or(a, b) => a.solve(variables) || b.solve(variables),
            Connective::Implicate(a, b) => (!a.solve(variables)) || b.solve(variables),
            Connective::Biimplicate(a, b) => a.solve(variables) == b.solve(variables),
            Connective::ForAll(_, _) => false, // todo
            Connective::Exists(_, _) => false, // todo
        }
    }

    fn symbol(&self) -> &'static str {
        match self {
            Connective::Var(_) | Connective::Predicate(_, _) => "",
            Connective::Not(_) => "¬",
            Connective::And(_, _) => "∧",
            Connective::Or(_, _) => "∨",
            Connective::Implicate(_, _) => "→",
            Connective::Biimplicate(_, _) => "↔",
            Connective::ForAll(_, _) => "∀",
            Connective::Exists(_, _) => "∃"
        }
    }

    fn latex_symbol(&self) -> &'static str {
        match self {
            Connective::Var(_) | Connective::Predicate(_, _) => "",
            Connective::Not(_) => "\\neg",
            Connective::And(_, _) => "\\land",
            Connective::Or(_, _) => "\\lor",
            Connective::Implicate(_, _) => "\\to",
            Connective::Biimplicate(_, _) => "\\leftrightarrow",
            Connective::ForAll(_, _) => "\\forall",
            Connective::Exists(_, _) => "\\exists",
        }
    }

    pub fn is_atomic(&self) -> bool {
        match self {
            Connective::Var(_) | Connective::Predicate(_, _) => true,
            _ => false,
        }
    }

    pub fn precedence(&self) -> usize {
        match self {
            Connective::Var(_) | Connective::Predicate(_, _) => 0,
            Connective::ForAll(_, _) => 1,
            Connective::Exists(_, _) => 1,
            Connective::Not(_) => 2,
            Connective::And(_, _) => 3,
            Connective::Or(_, _) => 4,
            Connective::Implicate(_, _) => 5,
            Connective::Biimplicate(_, _) => 5,
        }
    }

    pub fn associative(&self) -> bool {
        match self {
            Connective::Var(_) | Connective::Predicate(_, _) => true,
            Connective::Not(_) => true,
            Connective::And(_, _) => true,
            Connective::Or(_, _) => true,
            Connective::Implicate(_, _) => false,
            Connective::Biimplicate(_, _) => false,
            Connective::ForAll(_, _) => false,
            Connective::Exists(_, _) => false,
        }
    }

    pub fn pretty(&self) -> String {
        self.pretty_helper(9999)
    }

    fn pretty_helper(&self, precedence: usize) -> String {
        let own_precedence = self.precedence();
        let s = match self {
            Connective::Not(x) => format!("{}{}", self.symbol(), x.pretty_helper(own_precedence)),
            Connective::Predicate(x, y) => format!("{}({})", x, y.join(", ")),
            Connective::Var(x) => x.clone(),
            Connective::And(a, b)
            | Connective::Or(a, b)
            | Connective::Implicate(a, b)
            | Connective::Biimplicate(a, b) => format!(
                "{} {} {}",
                a.pretty_helper(own_precedence),
                self.symbol(),
                b.pretty_helper(own_precedence)
            ),
            Connective::ForAll(a, b) | Connective::Exists(a, b) => {
                format!("{}{} {}", self.symbol(), a, b.pretty_helper(own_precedence))
            }
        };
        if self.associative() && own_precedence <= precedence || own_precedence < precedence {
            s
        } else {
            format!("({})", s)
        }
    }

    pub fn pretty_latex(&self) -> String {
        self.pretty_latex_helper(9999)
    }

    fn pretty_latex_helper(&self, precedence: usize) -> String {
        let own_precedence = self.precedence();
        let s = match self {
            Connective::Not(x) => format!(
                "{} {}",
                self.latex_symbol(),
                x.pretty_latex_helper(own_precedence)
            ),
            Connective::Var(x) => x.clone(),
            Connective::Predicate(x, y) => format!("{}({})", x, y.join(", ")),
            Connective::And(a, b)
            | Connective::Or(a, b)
            | Connective::Implicate(a, b)
            | Connective::Biimplicate(a, b) => format!(
                "{} {} {}",
                a.pretty_latex_helper(own_precedence),
                self.latex_symbol(),
                b.pretty_latex_helper(own_precedence)
            ),
            Connective::ForAll(a, b) | Connective::Exists(a, b) => format!(
                "{}{} {}",
                self.latex_symbol(),
                a,
                b.pretty_latex_helper(own_precedence)
            ),
        };
        if self.associative() && own_precedence <= precedence || own_precedence < precedence {
            s
        } else {
            format!("({})", s)
        }
    }

    pub fn generate_table_generic<F>(&self, mut f: F) -> Table
    where
        F: FnMut(&Connective) -> String,
    {
        let variables = self.all_variables().into_iter().collect::<Vec<_>>();
        let permutations = all_permutations(&variables);
        let sub_connectives = self.all_sub_connectives(true);

        let headers = if let Some(headers) = permutations
            .iter()
            .map(|perm| {
                perm.keys()
                    .cloned()
                    .chain(sub_connectives.iter().map(|p| f(p)))
                    .collect()
            })
            .next() {
                headers
            } else {
                return Table { headers: vec![], rows: vec![] }
            };

        let rows = permutations
            .iter()
            .map(|perm| {
                perm.values()
                    .cloned()
                    .chain(sub_connectives.iter().map(|x| x.solve(perm)))
                    .collect()
            })
            .collect();

        Table { headers, rows }
    }

    pub fn generate_table_latex(&self) -> Table {
        self.generate_table_generic(|p| p.pretty_latex())
    }

    pub fn generate_table(&self) -> Table {
        self.generate_table_generic(|p| p.pretty())
    }

    pub fn substitude(&self, x: &str, y: &str) -> Connective {
        match self {
            Connective::Var(xx) if x == xx => Connective::Var(y.to_string()),
            Connective::Var(_) => self.clone(),
            Connective::Predicate(p, args) => Connective::Predicate(
                p.clone(),
                args.iter()
                    .map(|arg| {
                        if arg == x {
                            y.to_string()
                        } else {
                            arg.to_string()
                        }
                    })
                    .collect(),
            ),
            Connective::Not(i) => Connective::Not(box i.substitude(x, y)),
            Connective::And(a, b) => {
                Connective::And(box a.substitude(x, y), box b.substitude(x, y))
            }
            Connective::Or(a, b) => Connective::Or(box a.substitude(x, y), box b.substitude(x, y)),
            Connective::Implicate(a, b) => {
                Connective::Implicate(box a.substitude(x, y), box b.substitude(x, y))
            }
            Connective::Biimplicate(a, b) => {
                Connective::Biimplicate(box a.substitude(x, y), box b.substitude(x, y))
            }
            Connective::ForAll(xx, _) | Connective::Exists(xx, _) if xx == x => self.clone(),
            Connective::ForAll(xx, inner) => Connective::ForAll(xx.to_string(), box inner.substitude(x, y)),
            Connective::Exists(xx, inner) => Connective::Exists(xx.to_string(), box inner.substitude(x, y)),
        }
    }

    pub fn tableau_dot_graph(self, expect: bool) -> String {
        tableau::Tableau::new(vec![(self, expect)]).generate_dot()
    }

    pub fn table(&self) -> String {
        let table = self.generate_table_latex();

        format!(
            r#"\begin{{tabular}}{{|{}|}} \hline
{} \\ \hline
{} \\ \hline
\end{{tabular}}"#,
            table
                .headers
                .iter()
                .map(|_| "c")
                .collect::<Vec<_>>()
                .join("|"),
            indent(
                &table
                    .headers
                    .iter()
                    .map(|name| format!("${}$", name))
                    .collect::<Vec<_>>()
                    .join(" & ")
            ),
            indent(
                &table
                    .rows
                    .iter()
                    .map(|perm| perm
                        .iter()
                        .cloned()
                        .map(|value| if value { "T" } else { "F" })
                        .collect::<Vec<_>>()
                        .join(" & "))
                    .collect::<Vec<_>>()
                    .join(" \\\\ \\hline\n")
            )
        )
    }
}

fn indent(s: &str) -> String {
    s.split('\n')
        .map(|x| format!("\t{}", x))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<bool>>,
}

pub fn all_permutations(variables: &[String]) -> Vec<IndexMap<String, bool>> {
    match variables {
        [head, rest..] => {
            let variable = head.to_string();
            let mut permutations = vec![];
            for perm in all_permutations(rest) {
                let mut a = perm.clone();
                let mut b = perm;
                a.insert(variable.clone(), true);
                b.insert(variable.clone(), false);
                permutations.push(a);
                permutations.push(b);
            }
            permutations
        }
        [] => vec![IndexMap::new()],
    }
}
