type Con = Box<Connective>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Connective {
    Var(String),
    Predicate(String, Vec<String>),
    Not(Con),
    And(Con, Con),
    Or(Con, Con),
    Implicate(Con, Con),
    Biimplicate(Con, Con),
    ForAll(String, Con),
    Exists(String, Con),
    // All(Vec<Connective>),
    // Consequence(Con, Con),
}
