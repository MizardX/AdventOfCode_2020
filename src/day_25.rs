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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Handshake {
    card_pk: u64,
    door_pk: u64,
}

impl FromStr for Handshake {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let card_pk = lines.next().ok_or(ParseError::SyntaxError)?.parse()?;
        let door_pk = lines.next().ok_or(ParseError::SyntaxError)?.parse()?;
        if lines.next().is_some() {
            return Err(ParseError::SyntaxError);
        }
        Ok(Self { card_pk, door_pk })
    }
}

#[aoc_generator(day25)]
fn parse(input: &str) -> Result<Handshake, ParseError> {
    input.parse()
}

#[aoc(day25, part1)]
fn part_1(handshake: &Handshake) -> u64 {
    const MOD: u64 = 20_201_227;
    let mut door_exp = None;
    let mut card_exp = None;
    let mut val = 1;
    for exp in 0_u64..10_000_000 {
        if door_exp.is_none() && val == handshake.door_pk {
            door_exp = Some(exp);
            if card_exp.is_some() {
                break;
            }
        }
        if card_exp.is_none() && val == handshake.card_pk {
            card_exp = Some(exp);
            if door_exp.is_some() {
                break;
            }
        }
        val = (val * 7) % MOD;
    }
    if card_exp.is_none() || door_exp.is_none() {
        println!("Loop sizes not found");
        return 0;
    }
    let target_exp = card_exp.unwrap() * door_exp.unwrap() % (MOD - 1);
    let mut exp = target_exp;
    let mut base = 7;
    let mut res = 1;
    while exp > 0 {
        if exp & 1 == 0 {
            exp /= 2;
            base = (base * base) % MOD;
        } else {
            res = (res * base) % MOD;
            exp -= 1;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        5764801\n\
        17807724\
    ";

    #[test]
    fn test_part_1() {
        let handshake = parse(EXAMPLE).unwrap();
        let result = part_1(&handshake);
        assert_eq!(result, 14_897_079);
    }
}
