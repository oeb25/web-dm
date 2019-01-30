fn main() {
    let a = solver::parse(r#"(p & q) > r"#).unwrap();
    let b = solver::parse(r#"!(p > r)"#).unwrap();
    let c = solver::parse(r#"q > r"#).unwrap();
    let solved = solver::tableau::Tableau::new(vec![(a, true), (b, true), (c, false)]);
    println!("{}", solved.generate_dot());
}
