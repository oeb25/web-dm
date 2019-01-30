#[derive(Debug, Clone)]
pub struct Congruence {
    pub reductions: Vec<Vec<Quad>>,
    pub x_p: i64,
    pub n_p: i64,
}

#[derive(Debug)]
pub enum CongruenceError {
    NoSolution,
    DivideByZero,
}

pub fn congruence(a: i64, b: i64, n: i64) -> Result<Congruence, CongruenceError> {
    if a == 0 || b == 0 || n == 0 {
        return Err(CongruenceError::DivideByZero);
    }

    let extended = extended_euclid(n, a);

    let (r, _, t, _) = extended[extended.len() - 2];
    let gcd = r;

    if r == 1 {
        // n * s + a * t == 1
        // s * t % n == 1
        // x % n == n * t
        // x = x_p + n * z
        let x_p = b * t;
        Ok(Congruence {
            reductions: vec![extended],
            x_p,
            n_p: n
        })
    } else {
        if b % gcd != 0 {
            // sentence 5.7 does not hold
            return Err(CongruenceError::NoSolution);
        }
        let mut res = congruence(a / gcd, b / gcd, n / gcd)?;
        res.reductions.insert(0, extended);
        Ok(res)
    }
}

pub type Quad = (i64, i64, i64, i64);

pub fn extended_euclid(n: i64, m: i64) -> Vec<Quad> {
    let ast = (n, 1, 0, 0);
    let buv = (m, 0, 1, 0);

    let mut prev = vec![ast, buv];

    for i in 1.. {
        let a = prev[i - 1];
        let b = prev[i];

        if b.0 == 0 {
            return prev;
        }

        let q = a.0 / b.0;

        prev.push((
            a.0 - q * b.0,
            a.1 - q * b.1,
            a.2 - q * b.2,
            q,
        ));
    }

    unimplemented!();
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub fn chinese_rest_class(b_1: i64, n_1: i64, b_2: i64, n_2: i64) -> Result<(i64, i64), ()> {
    let extended = extended_euclid(n_1, n_2);

    let (r, s, t, _) = extended[extended.len() - 2];
    let gcd = r;

    if gcd != 1 {
        return Err(());
    }

    let (u_1, u_2) = (s, t);

    Ok((u_1, u_2))
}

// #[derive(Debug)]
// struct Polynomial {
//     coefficients: Vec<i64>,
// }

// impl Polynomial {
//     fn degree(&self) -> (usize, i64) {
//         self.coefficients.iter().cloned().enumerate().find(|(i, n)| *n == 0).unwrap_or((0, 0))
//     }
//     fn divide_by(&self, other: &Polynomial) {
//         let a = self.degree();
//         let b = other.degree();
//     }
//     fn mul_by_x(&self, n: usize) -> Polynomial {

//     }
// }

// fn euclid_polynomial(a: &[i64], b: &[i64]) {
//     let ast = (a, 1, 0, 0);
//     let buv = (b, 0, 1, 0);

//     let mut prev = vec![ast, buv];

//     for i in 1.. {
//         let a = prev[i - 1];
//         let b = prev[i];

//         if b.0 == 0 {
//             return prev;
//         }

//         let q = a.0 / b.0;

//         prev.push((
//             a.0 - q * b.0,
//             a.1 - q * b.1,
//             a.2 - q * b.2,
//             q,
//         ));
//     }

//     unimplemented!();
// }
