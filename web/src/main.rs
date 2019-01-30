#![feature(box_syntax, slice_patterns)]
#![recursion_limit = "256"]

#[macro_use]
extern crate stdweb;

use yew::{html, html_impl, prelude::*};

mod euclid;

#[derive(Debug)]
enum Msg {
    ChangeState(State),
    // Euclid
    GcdLeft(InputData),
    GcdRight(InputData),
    CongruenceA(InputData),
    CongruenceB(InputData),
    CongruenceN(InputData),
    ChiniseB1(InputData),
    ChiniseN1(InputData),
    ChiniseB2(InputData),
    ChiniseN2(InputData),
    ChangePolynomial(InputData, usize, usize),
    // Logic
    Change(InputData),
    Expect(bool),
    ShowSubSteps(String),
}

#[derive(Debug)]
struct LogicState {
    input: solver::Connective,
    dot_src: String,
    // latex_src: String,
    error: Option<solver::ParseError>,
    show_sub_steps: bool,
    expect: bool,
}

impl LogicState {
    fn new() -> LogicState {
        let start =
            "(b ∨ h) ∧ (h → ¬b) ∧ (¬h → ¬a) ∧ (l → k) ∧ (k → ¬b ∧ ¬a)";
        let parsed = solver::parse(start).expect("failed to parse");

        let expect = false;

        let new_dot = parsed.clone().tableau_dot_graph(expect);

        LogicState {
            input: parsed,
            dot_src: new_dot,
            error: None,
            show_sub_steps: false,
            expect: expect,
        }
    }
    fn redo_with(&mut self, value: &str) {
        match solver::parse(value) {
            Ok(parsed) => {
                self.input = parsed;
                self.redo();

                self.error = None;
            }
            Err(err) => self.error = Some(err),
        }
    }
    fn redo(&mut self) {
        let new_dot = self.input.clone().tableau_dot_graph(self.expect);
        self.dot_src = new_dot;
    }
}

const POLYNOMIAL_DEGREE: usize = 6;

#[derive(Debug)]
struct EuclidState {
    gcd: (i64, i64),
    congruence: (i64, i64, i64),
    congruence_result: Option<euclid::Congruence>,
    chinise_rest_class: (i64, i64, i64, i64),

    polynomial_a: [i64; POLYNOMIAL_DEGREE],
    polynomial_b: [i64; POLYNOMIAL_DEGREE],
}

impl EuclidState {
    fn new() -> EuclidState {
        EuclidState {
            gcd: (0,0),
            congruence: (0, 0, 0),
            congruence_result: None,
            chinise_rest_class: (0, 0, 0, 0),

            polynomial_a: [0; POLYNOMIAL_DEGREE],
            polynomial_b: [0; POLYNOMIAL_DEGREE],
        }
    }
    fn gcd(&self) -> Vec<euclid::Quad> {
        euclid::extended_euclid(self.gcd.0, self.gcd.1)
    }
}

#[derive(Debug)]
enum State {
    Logic(LogicState),
    Euclid(EuclidState),
}

struct Model {
    console: yew::services::console::ConsoleService,
    state: State,
}

impl Component<()> for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_props: (), _context: &mut Env<(), Model>) -> Model {
        Model {
            console: yew::services::console::ConsoleService::new(),
            state: State::Logic(LogicState::new()),
            // state: State::Euclid(EuclidState::new()),
        }
    }
    fn update(&mut self, msg: Msg, _context: &mut Env<(), Model>) -> bool {
        self.console.log(&format!("{:?}", msg));
        match msg {
            Msg::ChangeState(state) => {
                self.state = state;
                return true;
            }
            _ => {},
        }

        match &mut self.state {
            State::Euclid(state) => match msg {
                Msg::GcdLeft(data) => {
                    state.gcd.0 = data.value.parse().unwrap_or(0);
                }
                Msg::GcdRight(data) => {
                    state.gcd.1 = data.value.parse().unwrap_or(0);
                }
                Msg::CongruenceA(e) => state.congruence.0 = e.value.parse().unwrap_or(0),
                Msg::CongruenceB(e) => state.congruence.1 = e.value.parse().unwrap_or(0),
                Msg::CongruenceN(e) => state.congruence.2 = e.value.parse().unwrap_or(0),
                Msg::ChiniseB1(e) => state.chinise_rest_class.0 = e.value.parse().unwrap_or(0),
                Msg::ChiniseN1(e) => state.chinise_rest_class.1 = e.value.parse().unwrap_or(0),
                Msg::ChiniseB2(e) => state.chinise_rest_class.2 = e.value.parse().unwrap_or(0),
                Msg::ChiniseN2(e) => state.chinise_rest_class.3 = e.value.parse().unwrap_or(0),
                Msg::ChangePolynomial(e, i, n) => {
                    if i == 0 {
                        state.polynomial_a[n] = e.value.parse().unwrap_or(0);
                    } else {
                        state.polynomial_b[n] = e.value.parse().unwrap_or(0);
                    }
                }
                x => unimplemented!("{:?}", x),
            }
            State::Logic(logic) => match msg {
                Msg::Change(data) => self.redo_with(&data.value),
                Msg::Expect(expect) => {
                    logic.expect = expect;
                    self.redo();
                }
                Msg::ShowSubSteps(_value) => {
                    logic.show_sub_steps = !logic.show_sub_steps;
                }
                x => unimplemented!("{:?}", x),
            },
        }
        true
    }
}

impl Model {
    fn redo_with(&mut self, value: &str) {
        match &mut self.state {
            State::Logic(logic) => logic.redo_with(value),
            State::Euclid(_) => {},
        }
    }
    fn redo(&mut self) {
        match &mut self.state {
            State::Logic(logic) => logic.redo(),
            State::Euclid(_) => {},
        }
    }
}
// congruence
impl Renderable<(), Model> for Model {
    fn view(&self) -> Html<(), Model> {
        let inner = match &self.state {
            State::Euclid(state) => {
                let (a, b, n) = state.congruence;
                let res = euclid::congruence(a, b, n);
                let congruence_html = match &res {
                    Ok(result) => html! {
                        <div>{for result.reductions.iter().map(|r| html! {
                            <table>
                                <thead>
                                    <tr>
                                        {for ["k", "r", "s", "t", "q"].iter().map(|v| html! {<th>{v}</th>})}
                                    </tr>
                                </thead>
                                <tbody>
                                    {for r.iter().enumerate().map(|(k, (r, s, t, q))|
                                        html! {
                                            <tr>
                                                <td>{k}</td>
                                                <td>{r}</td>
                                                <td>{s}</td>
                                                <td>{t}</td>
                                                <td>{if *q == 0 {
                                                    "".to_string()
                                                } else {
                                                    format!("{}", q)
                                                }}</td>
                                            </tr>
                                        }
                                    )}
                                </tbody>
                            </table>
                        })}
                        <div>
                            {&format!("x = {} + {} * z", result.x_p, result.n_p)}
                        </div>
                        </div>
                    },
                    Err(e) => html! {<div>
                        {&format!("{:?}", e)}
                    </div>}
                };
                let (b_1, n_1, b_2, n_2) = state.chinise_rest_class;
                let chinise_rest_class_html = match euclid::chinese_rest_class(b_1, n_1, b_2, n_2) {
                    Ok(result) if n_1 != 0 && n_2 != 0 => html! {
                        <div>
                            <div>
                                {&format!("x ≡ {} * {} * {} + {} * {} * {} (mod {} * {})", result.0, n_1, b_2, result.1, n_2, b_1, n_1, n_2)}
                            </div>
                            <div>
                                {&format!("x ≡ {} (mod {})", result.0 * n_1 * b_2 + result.1 * n_2 * b_1, n_1 * n_2)}
                            </div>
                            <div>
                                {&format!("x ≡ {} (mod {})", (result.0 * n_1 * b_2 + result.1 * n_2 * b_1) % (n_1 * n_2), n_1 * n_2)}
                            </div>
                        </div>
                    },
                    Ok(_) => html! {<div/>},
                    Err(e) => html! {<div>
                        {&format!("{:?}", e)}
                    </div>}
                };
                let gcd_res = state.gcd();
                let gcd_row = gcd_res[gcd_res.len() - 2];
                html! {
                    <div>
                        <section>
                            <div>
                                {"gcd("}
                                <input type="number", oninput=|e| Msg::GcdLeft(e),/>
                                {","}
                                <input type="number", oninput=|e| Msg::GcdRight(e),/>
                                {format!(") = {} = {} * {} + {} * {}", gcd_row.0, state.gcd.0, gcd_row.1, state.gcd.1, gcd_row.2)}
                            </div>
                            <table>
                                <thead>
                                    <tr>
                                        {for ["k", "r", "s", "t", "q"].iter().map(|v| html! {<th>{v}</th>})}
                                    </tr>
                                </thead>
                                <tbody>
                                    {for gcd_res.into_iter().enumerate().map(|(k, (r, s, t, q))|
                                        html! {
                                            <tr>
                                                <td>{k}</td>
                                                <td>{r}</td>
                                                <td>{s}</td>
                                                <td>{t}</td>
                                                <td>{if q == 0 {
                                                    "".to_string()
                                                } else {
                                                    format!("{}", q)
                                                }}</td>
                                            </tr>
                                        }
                                    )}
                                </tbody>
                            </table>
                        </section>
                        <section>
                            <input type="number", oninput=|e| Msg::CongruenceA(e),/>
                            {" * x ≡ "}
                            <input type="number", oninput=|e| Msg::CongruenceB(e),/>
                            {"  (mod "}
                            <input type="number", oninput=|e| Msg::CongruenceN(e),/>
                            {")"}
                            {congruence_html}
                        </section>
                        <section>
                            <div>
                                {"x ≡ "}
                                <input type="number", oninput=|e| Msg::ChiniseB1(e),/>
                                {"  (mod "}
                                <input type="number", oninput=|e| Msg::ChiniseN1(e),/>
                                {")"}
                            </div>
                            <div>
                                {"x ≡ "}
                                <input type="number", oninput=|e| Msg::ChiniseB2(e),/>
                                {"  (mod "}
                                <input type="number", oninput=|e| Msg::ChiniseN2(e),/>
                                {")"}
                            </div>
                            {chinise_rest_class_html}
                        </section>
                        <section>
                            {for (0..2).map(|i| html! {
                                <div class="polynomial",>
                                    {for (0..POLYNOMIAL_DEGREE).map(|n| html!{
                                        <div>
                                            <input type="number", oninput=|e| Msg::ChangePolynomial(e, i, n),/>
                                            {if n + 1 == POLYNOMIAL_DEGREE {
                                                "".to_string()
                                            } else {
                                                format!("x^{}", POLYNOMIAL_DEGREE - n + 1)
                                            }}
                                        </div>
                                    })}
                                    {format!("{:?}", (&state.polynomial_a, &state.polynomial_b))}
                                </div>
                            })}
                            <div>
                                {"x ≡ "}
                                <input type="number", oninput=|e| Msg::ChiniseB2(e),/>
                                {"  (mod "}
                                <input type="number", oninput=|e| Msg::ChiniseN2(e),/>
                                {")"}
                            </div>
                        </section>
                    </div>
                }
            }
            State::Logic(logic) => {
                let table = logic.input.generate_table();

                let dot_src = logic.dot_src.clone();
                js! {
                    tryDraw(@{dot_src});
                };

                let expect = logic.expect.clone();

                let expect_str = format!("Expecting {}", if expect { "true" } else { "false" });

                html! {
                    <div>
                        <textarea oninput=|e| Msg::Change(e), />
                        <input type="checkbox", checked={logic.show_sub_steps}, oninput=|e| {
                            Msg::ShowSubSteps(e.value)
                        },/>
                        <button onclick=|_| Msg::Expect(!expect), >{expect_str}</button>
                        <details>
                            <summary>{"Table"}</summary>
                            <table>
                                <thead>
                                    <tr>
                                        {for table.headers.iter().map(|name| html!{
                                            <th>{name}</th>
                                        })}
                                    </tr>
                                </thead>
                                <tbody>
                                    {for table.rows.iter().map(|perm| html!{
                                        <tr>
                                            {for perm.iter().cloned().map(|value|
                                                if value {
                                                    html!{<td style="text-align: center;",>{"T"}</td>}
                                                } else {
                                                    html!{<td style="text-align: center; background: gray; color: white",>{"F"}</td>}
                                                }
                                            )}
                                        </tr>
                                    })}
                                </tbody>
                            </table>
                        </details>
                        <details>
                            <summary>{"Latex Table"}</summary>
                            <pre><code>
                                {logic.input.table()}
                            </code></pre>
                        </details>
                    </div>
                }
            }
        };

        html! {
            <div>
                <div>
                    <button onclick=|_| Msg::ChangeState(State::Logic(LogicState::new())),>{"Logic"}</button>
                    <button onclick=|_| Msg::ChangeState(State::Euclid(EuclidState::new())),>{"Euclid"}</button>
                </div>
                {inner}
            </div>
        }
    }
}

fn main() {
    use stdweb::web::{document, IParentNode};

    yew::initialize();
    let mount = document().query_selector("#app").unwrap().unwrap();

    App::<(), Model>::new(()).mount(mount);
    yew::run_loop();
}
