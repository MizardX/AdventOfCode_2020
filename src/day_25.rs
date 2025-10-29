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
const fn part_1(handshake: &Handshake) -> u64 {
    const MOD: u64 = 20_201_227;
    let mut public_key = 1;
    let mut encryption_key = [1; 2];
    loop {
        public_key = public_key * 7 % MOD;
        encryption_key[0] = encryption_key[0] * handshake.door_pk % MOD;
        if public_key == handshake.card_pk {
            return encryption_key[0];
        }
        // This is redundant, but only adds 3,9% execution time.
        // Doing it the other way around, by removing the `encryption_key[0]`
        // part triples the excution time though. Better to keep both, so it
        // works almost fastest for all inputs.
        encryption_key[1] = encryption_key[1] * handshake.card_pk % MOD;
        if public_key == handshake.door_pk {
            return encryption_key[1];
        }
    }
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
