use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Number(u64),
    Operator(Operator),
    OpenParen,
    CloseParen,
}

impl FromStr for Token {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "+" => Self::Operator(Operator::Plus),
            "*" => Self::Operator(Operator::Times),
            "(" => Self::OpenParen,
            ")" => Self::CloseParen,
            _ if s.bytes().all(|b| b.is_ascii_digit()) => Self::Number(s.parse()?),
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Plus,
    Times,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Expression {
    tokens: Vec<Token>,
}

#[derive(Debug, Error)]
enum EvaluationError {
    #[error("Unexpected number {0} in expression")]
    UnexpectedNumber(u64),
    #[error("Unexpected operator {0:?} in expression")]
    UnexpectedOperator(Operator),
    #[error("Unexpected '(' in expression")]
    UnexpectedOpenParen,
    #[error("Unexpected ')' in expression")]
    UnexpectedCloseParen,
    #[error("Unclosed parens")]
    UnclosedParens,
    #[error("Unexpected end of expression")]
    UnexpectedEnd,
}

impl Expression {
    fn evaluate<RULES>(&self) -> Result<u64, EvaluationError>
    where
        RULES: EvaluationRules,
        RULES::State: Copy,
    {
        let mut state = None;
        let mut operator = None;
        let mut stack = Vec::new();
        for &tok in &self.tokens {
            match tok {
                Token::Number(num) => {
                    state = Some(match (state, operator) {
                        (None, None) => RULES::start(num),
                        (Some(state), Some(op)) => RULES::evaluate(state, op, num),
                        _ => return Err(EvaluationError::UnexpectedNumber(num)),
                    });
                    operator = None;
                }
                Token::Operator(op) => {
                    operator = match (state, operator) {
                        (Some(_), None) => Some(op),
                        _ => return Err(EvaluationError::UnexpectedOperator(op)),
                    };
                }
                Token::OpenParen => match (state, operator) {
                    (None, None) | (Some(_), Some(_)) => {
                        stack.push((state, operator));
                        state = None;
                        operator = None;
                    }
                    _ => return Err(EvaluationError::UnexpectedOpenParen),
                },
                Token::CloseParen => {
                    let num = match (state, operator) {
                        (Some(state), None) => RULES::finish(state),
                        _ => return Err(EvaluationError::UnexpectedCloseParen),
                    };

                    state = Some(match stack.pop() {
                        Some((Some(state), Some(op))) => RULES::evaluate(state, op, num),
                        Some((None, None)) => RULES::start(num),
                        _ => return Err(EvaluationError::UnexpectedCloseParen),
                    });
                    operator = None;
                }
            }
        }
        match (stack.is_empty(), state, operator) {
            (false, _, _) => Err(EvaluationError::UnclosedParens),
            (_, Some(state), None) => Ok(RULES::finish(state)),
            _ => Err(EvaluationError::UnexpectedEnd),
        }
    }
}

impl FromStr for Expression {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s
            .split_inclusive(['+', '*', '(', ')'])
            .flat_map(|word| {
                let n = word.len();
                let (prefix, op) = word.split_at(n - 1);
                if op.ends_with(['+', '*', '(', ')']) {
                    let prefix = prefix.trim_ascii();
                    if prefix.is_empty() {
                        [None, Some(op.parse())]
                    } else {
                        [Some(prefix.parse()), Some(op.parse())]
                    }
                } else {
                    let prefix = word.trim_ascii();
                    if prefix.is_empty() {
                        [None, None]
                    } else {
                        [Some(prefix.parse()), None]
                    }
                }
            })
            .flatten()
            .collect::<Result<_, _>>()?;
        Ok(Self { tokens })
    }
}

trait EvaluationRules {
    type State;
    fn start(val: u64) -> Self::State;
    fn evaluate(state: Self::State, op: Operator, val: u64) -> Self::State;
    fn finish(state: Self::State) -> u64;
}

struct FlatRules;
impl EvaluationRules for FlatRules {
    type State = u64;

    fn start(val: u64) -> Self::State {
        val
    }

    fn evaluate(state: Self::State, op: Operator, val: u64) -> Self::State {
        match op {
            Operator::Plus => state + val,
            Operator::Times => state * val,
        }
    }

    fn finish(state: Self::State) -> u64 {
        state
    }
}

struct TimesBeforePlus;
impl EvaluationRules for TimesBeforePlus {
    type State = (u64, u64);

    fn start(val: u64) -> Self::State {
        (val, 1)
    }

    fn evaluate((sum, product): Self::State, op: Operator, val: u64) -> Self::State {
        match op {
            Operator::Plus => (sum + val, product),
            Operator::Times => (val, sum * product),
        }
    }

    fn finish((sum, product): Self::State) -> u64 {
        sum * product
    }
}

#[aoc_generator(day18)]
fn parse(input: &str) -> Result<Vec<Expression>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day18, part1)]
fn part_1(expressions: &[Expression]) -> u64 {
    expressions
        .iter()
        .map(Expression::evaluate::<FlatRules>)
        .sum::<Result<u64, EvaluationError>>()
        .unwrap()
}

#[aoc(day18, part2)]
fn part_2(expressions: &[Expression]) -> u64 {
    expressions
        .iter()
        .map(Expression::evaluate::<TimesBeforePlus>)
        .sum::<Result<u64, EvaluationError>>()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "1 + 2 * 3 + 4 * 5 + 6";
    const EXAMPLE2: &str = "1 + (2 * 3) + (4 * (5 + 6))";
    const EXAMPLE3: &str = "2 * 3 + (4 * 5)";
    const EXAMPLE4: &str = "5 + (8 * 3 + 9 + 3 * 4 * 3)";
    const EXAMPLE5: &str = "5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))";
    const EXAMPLE6: &str = "((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE3).unwrap();
        assert_eq!(
            result[0].tokens,
            [
                Token::Number(2),
                Token::Operator(Operator::Times),
                Token::Number(3),
                Token::Operator(Operator::Plus),
                Token::OpenParen,
                Token::Number(4),
                Token::Operator(Operator::Times),
                Token::Number(5),
                Token::CloseParen
            ]
        );
    }

    #[test_case(EXAMPLE1 => 71)]
    #[test_case(EXAMPLE2 => 51)]
    #[test_case(EXAMPLE3 => 26)]
    #[test_case(EXAMPLE4 => 437)]
    #[test_case(EXAMPLE5 => 12_240)]
    #[test_case(EXAMPLE6 => 13_632)]
    fn test_part_1(input: &str) -> u64 {
        let tokens = parse(input).unwrap();
        part_1(&tokens)
    }

    #[test_case(EXAMPLE1 => 231)]
    #[test_case(EXAMPLE2 => 51)]
    #[test_case(EXAMPLE3 => 46)]
    #[test_case(EXAMPLE4 => 1_445)]
    #[test_case(EXAMPLE5 => 669_060)]
    #[test_case(EXAMPLE6 => 23_340)]
    fn test_part_2(input: &str) -> u64 {
        let tokens = parse(input).unwrap();
        part_2(&tokens)
    }
}
