use crate::ast::Connective;
use crate::parse;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
struct Node {
    connectives: Vec<(FactId, Connective, bool)>,
    closed: bool,
}

#[derive(Debug, Clone)]
struct Edge {
    origin_node: NodeId,
    fact: FactId,
    to: NodeId,
}

#[derive(Debug, Clone, Copy)]
struct NodeId(usize);

#[derive(Debug, Clone, Copy)]
struct FactId(usize);

#[derive(Debug, Clone)]
enum QueueEntry {
    Repeated(usize, FactId, Connective, bool, String, bool),
    Standard(usize, FactId, Connective, bool),
}

impl QueueEntry {
    fn extract(&self) -> (&Connective, bool, Option<&str>) {
        match self {
            QueueEntry::Standard(_, _, con, expect) => (con, *expect, None),
            QueueEntry::Repeated(_, _, con, expect, to_repace, _) => (con, *expect, Some(to_repace)),
        }
    }
}

#[derive(Debug, Clone)]
struct Knowlage {
    facts: HashMap<Connective, bool>,
    queue: VecDeque<(FactId, Connective, bool)>,
    known_constants: HashSet<String>,
    repeaters: Vec<(FactId, String, Connective, bool, HashSet<String>)>,
}

impl Knowlage {
    fn new(known_constants: HashSet<String>) -> Knowlage {
        Knowlage {
            queue: Default::default(),
            facts: Default::default(),
            known_constants,
            repeaters: vec![],
        }
    }
    fn generate_queue(&self) -> Vec<QueueEntry> {
        let queued = self
            .queue
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, (fact_id, connective, expect))| {
                QueueEntry::Standard(i, fact_id, connective, expect)
            });
        let repeated = self.repeaters.iter().enumerate().flat_map(
            |(i, (fact_id, to_repalce, con, expect, ran_on))| {
                // if self.known_constants.is_empty() {
                //     vec![QueueEntry::Repeated(i, *fact_id, con.clone(), *expect, to_repalce.to_string(), true)]
                // } else {
                    self.known_constants
                        .difference(&ran_on)
                        .map(move |constant| {
                            QueueEntry::Repeated(
                                i,
                                *fact_id,
                                con.substitude(&to_repalce, &constant),
                                *expect,
                                constant.to_string(),
                                false,
                            )
                        })
                // }
            },
        );
        queued.chain(repeated).collect()
    }
    fn process_queue_entry(&mut self, entry: QueueEntry) -> (FactId, Connective, bool, bool) {
        match entry {
            QueueEntry::Repeated(index, fact_id, connective, expect, constant, introduce_constant) => {
                if introduce_constant {
                    self.known_constants.insert(constant.to_string());
                }

                if let Some(repeater) = self.repeaters.get_mut(index) {
                    repeater.4.insert(constant);
                } else {
                    panic!("repeater did not exists");
                }
                (fact_id, connective, expect, true)
            }
            QueueEntry::Standard(index, fact_id, connective, expect) => {
                self.queue.remove(index);
                (fact_id, connective, expect, false)
            }
        }
    }
    fn pop(&mut self) -> Option<(FactId, Connective, bool, bool)> {
        let queue = self.generate_queue();


        let entry = queue.clone().into_iter().min_by_key(|entry| {
            let (con, expect, repeated) = entry.extract();

            let contra = |con, expect: bool| self.facts.get(con).cloned() == Some(!expect);

            match (con, expect, repeated) {
                (Connective::Var(_), _, _) => 1,
                (Connective::And(a, b), true, _) if contra(a, true) || contra(b, true) => 0,
                (Connective::Or(a, b), false, _) if contra(a, false) || contra(b, false) => 0,
                (Connective::And(_, _), true, _) |
                (Connective::Or(_, _), false, _) => 2,
                (Connective::Implicate(a, b), false, _) if contra(a, true) || contra(b, false) => 0,
                (Connective::Implicate(_, _), false, _) => 3,
                (Connective::Implicate(a, b), true, _) if contra(a, false) || contra(b, true) => 0,
                _ => 100
            }
        })?;
        // let entry = queue.into_iter().next()?;

        // let pretty_entry = |entry: &QueueEntry| {
        //     match entry {
        //         QueueEntry::Standard(_, _, con, expect) => format!("S({}: {})", con.pretty(), expect),
        //         QueueEntry::Repeated(_, _, con, expect, _, _) => format!("R({}: {})", con.pretty(), expect),
        //     }
        // };

        // println!("{:?}, choose {:?}", queue.iter().map(pretty_entry).collect::<Vec<_>>().join(", "), pretty_entry(&entry));

        Some(self.process_queue_entry(entry))
    }
    fn fact(&mut self, connective: Connective, expect: bool) -> Result<bool, FactResult> {
        if let Some(fact) = self.facts.get(&connective) {
            if expect == *fact {
                Ok(false)
            } else {
                Err(FactResult::Closes)
            }
        } else {
            self.facts.insert(connective, expect);
            Ok(true)
        }
    }
    fn queue(&mut self, fact_id: FactId, connective: Connective, expect: bool) {
        self.queue.push_back((fact_id, connective, expect));
    }
    fn add_repeater(
        &mut self,
        fact_id: FactId,
        to_repalce: String,
        connective: Connective,
        expect: bool,
    ) {
        self.repeaters
            .push((fact_id, to_repalce, connective, expect, HashSet::new()))
    }
    fn register_constant(&mut self, constant: String) {
        self.known_constants.insert(constant);
    }
}

#[derive(Debug, Clone, PartialEq)]
enum FactResult {
    Closes,
}

#[derive(Debug, Clone)]
pub struct Tableau {
    facts_counter: usize,
    constant_counter: usize,
    process_counter: usize,
    knowlage: Knowlage,
    knowlage_stack: Vec<Knowlage>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Tableau {
    pub fn new(start: Vec<(Connective, bool)>) -> Tableau {
        let known_constants: HashSet<_> = start
            .iter()
            .flat_map(|(con, _)| con.all_variables().into_iter())
            .collect();

        let mut tableau = Tableau {
            facts_counter: 0,
            constant_counter: 0,
            process_counter: 0,
            knowlage: Knowlage::new(known_constants),
            knowlage_stack: vec![],
            nodes: vec![],
            edges: vec![],
        };

        let (staring_node_id, node) = tableau.alloc_node(start.clone());
        let connectives = node.connectives.clone();
        let result = tableau.queue_facts(connectives);
        tableau.process_next(staring_node_id, result);
        tableau
    }
    fn queue_facts(
        &mut self,
        connectives: impl IntoIterator<Item = (FactId, Connective, bool)>,
    ) -> Result<(), FactResult> {
        for (fact_id, con, expect) in connectives {
            if self.knowlage.fact(con.clone(), expect)? {
                self.knowlage.queue(fact_id, con, expect);
            }
        }
        Ok(())
    }
    fn pop_queue(&mut self) -> Option<(FactId, Connective, bool, bool)> {
        self.knowlage.pop()
    }
    fn create_edge(&mut self, origin_node: NodeId, fact: FactId, to: NodeId) {
        self.edges.push(Edge {
            origin_node,
            fact,
            to,
        });
    }
    fn save_knowlage(&mut self) {
        let save_point = self.knowlage.clone();
        self.knowlage_stack.push(save_point);
    }
    fn restore_knowlage(&mut self) {
        assert!(self.knowlage_stack.len() > 0);
        self.knowlage = self.knowlage_stack.pop().expect("knowlage_stack was empty");
    }
    fn alloc_node(
        &mut self,
        connectives: impl IntoIterator<Item = (Connective, bool)>,
    ) -> (NodeId, &Node) {
        let connectives = connectives
            .into_iter()
            .map(|(con, expect)| {
                let fact_id = self.facts_counter;
                self.facts_counter += 1;
                (FactId(fact_id), con.clone(), expect)
            })
            .collect();

        let node_id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            connectives,
            closed: false,
        });
        (node_id, &self.nodes[node_id.0])
    }
    fn alloc_constant(&mut self) -> String {
        let con = format!("C{}", self.constant_counter);
        self.knowlage.register_constant(con.clone());
        self.constant_counter += 1;
        // println!("{:?}", self.constant_counter);
        con
    }
    fn process_next(&mut self, from: NodeId, last_result: Result<(), FactResult>) -> bool {
        if self.process_counter >= 300 {
            // println!("Exeed process limit");
            return false;
        }
        if self.constant_counter >= 100 {
            return false;
        }
        if self.facts_counter >= 100 {
            // println!("more then 100 facts");
            return false;
        }
        self.process_counter += 1;
        match last_result {
            Ok(()) => {
                if let Some((fact, con, expect, was_repeated)) = self.pop_queue() {
                    if was_repeated {
                        self.straight(from, fact, vec![(con, expect)]);
                    } else {
                        self.process(from, fact, con, expect);
                    }
                } else {
                    // println!("Queue was empty!");
                }
            }
            Err(FactResult::Closes) => {
                self.nodes[from.0].closed = true;
            }
        }
        true
    }
    fn process(
        &mut self,
        from: NodeId,
        fact_id: FactId,
        connective: Connective,
        expect: bool,
    ) -> bool {
        match connective {
            Connective::Var(_) | Connective::Predicate(_, _) => self.process_next(from, Ok(())),
            Connective::And(left, right) => {
                if expect {
                    self.straight(from, fact_id, vec![(*left, true), (*right, true)])
                } else {
                    self.branch(from, fact_id, vec![(*left, false)], vec![(*right, false)])
                }
            }
            Connective::Or(left, right) => {
                if expect {
                    self.branch(from, fact_id, vec![(*left, true)], vec![(*right, true)])
                } else {
                    self.straight(from, fact_id, vec![(*left, false), (*right, false)])
                }
            }
            Connective::Implicate(left, right) => {
                if expect {
                    self.branch(from, fact_id, vec![(*left, false)], vec![(*right, true)])
                } else {
                    self.straight(from, fact_id, vec![(*left, true), (*right, false)])
                }
            }
            Connective::Biimplicate(left, right) => {
                if expect {
                    self.branch(
                        from,
                        fact_id,
                        vec![(*left.clone(), false), (*right.clone(), false)],
                        vec![(*left, true), (*right, true)],
                    )
                } else {
                    self.branch(
                        from,
                        fact_id,
                        vec![(*left.clone(), false), (*right.clone(), true)],
                        vec![(*left, true), (*right, false)],
                    )
                }
            }
            Connective::Not(con) => self.straight(from, fact_id, vec![(*con, !expect)]),
            Connective::Exists(var, con) => {
                if expect {
                    let new_const = self.alloc_constant();
                    let new_con = con.substitude(&var, &new_const);
                    self.straight(from, fact_id, vec![(new_con, expect)])
                } else {
                    self.knowlage
                        .add_repeater(fact_id, var.clone(), *con.clone(), expect);
                    self.straight(from, fact_id, vec![])
                }
            }
            Connective::ForAll(var, con) => {
                if expect {
                    self.knowlage
                        .add_repeater(fact_id, var.clone(), *con.clone(), expect);
                    self.straight(from, fact_id, vec![])
                } else {
                    let new_const = self.alloc_constant();
                    let new_con = con.substitude(&var, &new_const);
                    self.straight(from, fact_id, vec![(new_con, expect)])
                }
            }
        }
    }
    fn straight(&mut self, from: NodeId, fact_id: FactId, cons: Vec<(Connective, bool)>) -> bool {
        if !cons.is_empty() {
            let (node_id, node) = self.alloc_node(cons);
            let connectives = node.connectives.clone();
            self.create_edge(from, fact_id, node_id);
            let result = self.queue_facts(connectives);
            self.process_next(node_id, result);
        } else {
            self.process_next(from, Ok(()));
        }
        true
    }
    fn branch(
        &mut self,
        from: NodeId,
        fact_id: FactId,
        left: Vec<(Connective, bool)>,
        right: Vec<(Connective, bool)>,
    ) -> bool {
        self.save_knowlage();
        self.straight(from, fact_id, left);
        self.restore_knowlage();
        self.save_knowlage();
        self.straight(from, fact_id, right);
        self.restore_knowlage();
        true
    }
    pub fn generate_dot(&self) -> String {
        let header = "digraph A {\n\t";
        let footer = "\t\n}";

        let nodes = self
            .nodes
            .iter()
            .enumerate()
            .map(|(id, node)| {
                let mut label = node
                    .connectives
                    .iter()
                    .map(|(id, con, expect)| {
                        format!(r#"[{}] {}: {}"#, id.0 + 1, con.pretty(), expect)
                    })
                    .collect::<Vec<_>>()
                    .join("\\n");
                if node.closed {
                    label += "\\nx";
                }
                format!(r#"{} [label="{}"];"#, id, label)
            })
            .collect::<Vec<_>>()
            .join("\n\t");

        let edges = self
            .edges
            .iter()
            .map(|edge| {
                format!(
                    "{} -> {} [label=\"{}\"];",
                    edge.origin_node.0,
                    edge.to.0,
                    edge.fact.0 + 1
                )
            })
            .collect::<Vec<_>>()
            .join("\n\t");

        format!("{}{}\n\n\t{}{}", header, nodes, edges, footer)
    }
}

#[allow(unused)]
fn run(src: &str, expect: bool) -> Tableau {
    let con = parse::parse(src).unwrap();
    Tableau::new(vec![(con, expect)])
}

#[test]
fn simple_or() {
    // println!("{}", run("P(a) | b", false).generate_dot());
    // println!("{}", run("a | (b | a)", false).generate_dot());
    // println!("{}", run("a | (b & a)", false).generate_dot());
    // println!("{}", run("a > b & (x = !a)", false).generate_dot());
    println!(
        "{}",
        run(
            "(b ∨ h) ∧ (h → ¬b) ∧ (¬h → ¬a) ∧ (l → k) ∧ (k → ¬b ∧ ¬a)",
            false
        )
        .generate_dot()
    );
    // println!("{}", run(".a (a | b)", false).generate_dot());
    // println!("{}", run(".a a > b | c", false).generate_dot());

    // assert!(false);
}
//     let con = parse::parse(r#"(A & !B) & (B & !A) = (A & B) & !(A | B)"#).unwrap();

#[test]
fn other_test() {
    let a = parse::parse(r#"(\x(\y(P(x,y) > P(y,x))))"#).unwrap();
    // let b = parse::parse(r#"(\x(\y(P(x,y) = P(y,x))))"#).unwrap();
    println!(
        "{}",
        Tableau::new(vec![(a, true)]).generate_dot()
    );
    unimplemented!();
}

// #[test]
// fn other_equiv_test() {
//     let a = parse::parse("(\\x(.y(P(x, y))))").unwrap();
//     let b = parse::parse("(.z(P(z, z)))").unwrap();
//     // let c = parse::parse("!q").unwrap();
    // println!(
//         "{}",
//         Tableau::new(vec![(a, true), (b, false)]).generate_dot()
//     );
//     // unimplemented!();
// }

// #[test]
// fn fails_in_browser_index_out_of_bounds() {
//     println!("start");
//     println!("{}", run("(\\x(.y (P(y,x)))) > a", false).generate_dot());
//     println!("end");
// }
