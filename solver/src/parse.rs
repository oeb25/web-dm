use crate::ast::Connective;

#[derive(Debug, Clone)]
pub enum Token {
    Var(String),
    Not,
    And,
    Or,
    Implicate,
    Biimplicate,
    Comma,
    Slash,
    OpenParen,
    CloseParen,
    ForAll,
    Exists,
}

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = vec![];

    for (_i, c) in src.char_indices() {
        match c {
            '¬' | '!' => tokens.push(Token::Not),
            '∧' | '&' | '∪' => tokens.push(Token::And),
            '∨' | '|' | '∩' => tokens.push(Token::Or),
            '→' | '>' => tokens.push(Token::Implicate),
            '↔' | '=' => tokens.push(Token::Biimplicate),
            ',' => tokens.push(Token::Comma),
            '/' => tokens.push(Token::Slash),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '\\' => tokens.push(Token::ForAll),
            '.' => tokens.push(Token::Exists),
            ' ' => {}
            c => tokens.push(Token::Var(c.to_string())),
        }
    }

    tokens
}

#[derive(Debug)]
pub enum ParseError {
    UncosedParen(Option<Token>),
    UnexpectedToken(Option<Token>),
    InvalidArgumentList(Option<Token>),
}

type ParseResult<T> = Result<T, ParseError>;

fn parse_top(tokens: &[Token]) -> ParseResult<(Connective, &[Token])> {
    parse3(tokens)
}
/*
fn parse3(tokens: &[Token]) -> ParseResult<(Connective, &[Token])> {
    let (left, rest) = parse3(tokens)?;

    match rest {
        [Token::Comma, rest..] => {
            let mut args = vec![left];
            let mut rest = rest;
            loop {
                let (right, inner_rest) = parse3(rest)?;
                args.push(right);
                match inner_rest {
                    [Token::Comma, inner_rest..] => {
                        rest = inner_rest;
                    }
                    [Token::Slash, rest..] => {
                        let (right, rest) = parse3(rest)?;
                        let predicate = Connective::All(args);
                        return Ok((Connective::Consequence(box predicate, box right), rest));
                    }
                    [] => {
                        return Ok((Connective::All(args), &[]));
                    }
                    [x, _..] => return Err(ParseError::UnexpectedToken(Some(x.clone()))),
                }
            }
        }
        [Token::Slash, rest..] => {
            let (right, rest) = parse3(rest)?;
            return Ok((Connective::Consequence(box left, box right), rest));
        }
        _ => Ok((left, rest)),
    }
}
*/

fn parse3(tokens: &[Token]) -> ParseResult<(Connective, &[Token])> {
    let (left, rest) = parse2(tokens)?;
    Ok(match rest {
        [Token::Implicate, rest..] => {
            let (right, rest) = parse3(rest)?;
            (Connective::Implicate(box left, box right), rest)
        }
        [Token::Biimplicate, rest..] => {
            let (right, rest) = parse3(rest)?;
            (Connective::Biimplicate(box left, box right), rest)
        }
        _ => (left, rest),
    })
}

fn parse2(tokens: &[Token]) -> ParseResult<(Connective, &[Token])> {
    let (left, rest) = parse1(tokens)?;

    Ok(match rest {
        [Token::And, rest..] => {
            let (right, rest) = parse2(rest)?;
            (Connective::And(box left, box right), rest)
        }
        [Token::Or, rest..] => {
            let (right, rest) = parse2(rest)?;
            (Connective::Or(box left, box right), rest)
        }
        _ => (left, rest),
    })
}

fn parse1(tokens: &[Token]) -> ParseResult<(Connective, &[Token])> {
    Ok(match tokens {
        [Token::Not, rest..] => {
            let (expr, rest) = parse1(rest)?;
            (Connective::Not(box expr), rest)
        }
        [Token::ForAll, name, rest..] => {
            let name = if let Token::Var(name) = name {
                name
            } else {
                return Err(ParseError::UnexpectedToken(Some(name.clone())));
            };

            let (right, rest) = parse1(rest)?;

            (Connective::ForAll(name.clone(), box right), rest)
        }
        [Token::Exists, name, rest..] => {
            let name = match name {
                Token::Var(name) => name,
                x => Err(ParseError::UnexpectedToken(Some(x.clone())))?,
            };

            let (right, rest) = parse1(rest)?;

            (Connective::Exists(name.clone(), box right), rest)
        }
        [Token::Var(x), Token::OpenParen, Token::Var(arg), rest..] => {
            let mut rest = rest;
            let mut args = vec![arg.clone()];
            loop {
                match rest {
                    [Token::Comma, Token::Var(arg), nrest..] => {
                        args.push(arg.clone());
                        rest = nrest;
                    }
                    [Token::CloseParen, nrest..] => {
                        rest = nrest;
                        break;
                    }
                    x => return Err(ParseError::InvalidArgumentList(x.get(0).cloned())),
                }
            }
            (Connective::Predicate(x.clone(), args), rest)
        }
        [Token::Var(x), rest..] => (Connective::Var(x.to_string()), rest),
        [Token::OpenParen, rest..] => {
            let (expr, rest) = parse_top(rest)?;
            match rest {
                [Token::CloseParen, rest..] => (expr, rest),
                x => return Err(ParseError::UncosedParen(x.get(0).cloned())),
            }
        }
        x => return Err(ParseError::UnexpectedToken(x.get(0).cloned())),
    })
}

pub fn parse(src: &str) -> ParseResult<Connective> {
    let tokens = lex(src);
    let (con, _) = parse_top(&tokens)?;
    Ok(con)
}

#[test]
fn out_of_bounds_index() {
    assert!(parse(".c (.a a > b)").is_ok())
}
