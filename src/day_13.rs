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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Input {
    earliest_departure: u64,
    schedule: Vec<Bus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bus {
    id: u64,
    offset: u64,
}

impl FromStr for Input {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let earliset_departure = lines.next().ok_or(ParseError::SyntaxError)?.parse()?;
        let schedule = lines
            .next()
            .ok_or(ParseError::SyntaxError)?
            .split(',')
            .zip(0..)
            .filter_map(|(id, offset)| {
                if id == "x" {
                    None
                } else {
                    Some(id.parse().map(|id| Bus { id, offset }))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        if lines.next().is_some() {
            return Err(ParseError::SyntaxError);
        }
        Ok(Self {
            earliest_departure: earliset_departure,
            schedule,
        })
    }
}

#[aoc_generator(day13)]
fn parse(input: &str) -> Result<Input, ParseError> {
    input.parse()
}

#[aoc(day13, part1)]
fn part_1(input: &Input) -> u64 {
    let (delay, id) = input
        .schedule
        .iter()
        .map(|bus| {
            (
                (bus.id - input.earliest_departure % bus.id) % bus.id,
                bus.id,
            )
        })
        .min()
        .unwrap();
    delay * id
}

#[aoc(day13, part2)]
fn part_2(input: &Input) -> u64 {
    // t + bus.offset === 0 (mod bus.id)
    // t === -bus.offset (mod bus.id)
    input
        .schedule
        .iter()
        .copied()
        // time + offset === 0 (mod id)
        // time === -offset (mod id)
        // time === id - offset (mod id)
        .map(|bus| ((bus.id - bus.offset % bus.id) % bus.id, bus.id))
        .reduce(|(value1, mod1), (value2, mod2)| {
            (chinese_remainder(value1, mod1, value2, mod2), mod1 * mod2)
        })
        .unwrap()
        .0
}

fn chinese_remainder(value1: u64, mod1: u64, value2: u64, mod2: u64) -> u64 {
    let (gcd, bez1, bez2) = extended_gcd(mod1, mod2);
    assert_eq!(gcd, 1, "Must be coprime");
    // Have to use i128 because of multiplication overflow, but the result is < mod1*mod2
    let signed = i128::from(value1) * i128::from(mod2) * i128::from(bez2)
        + i128::from(value2) * i128::from(mod1) * i128::from(bez1);
    signed
        .rem_euclid(i128::from(mod1) * i128::from(mod2))
        .try_into()
        .unwrap()
}

pub const fn extended_gcd(mut x: u64, mut y: u64) -> (u64, i64, i64) {
    let (mut xa, mut xb) = (1, 0);
    let (mut ya, mut yb) = (0, 1);

    while let Some(rem) = x.checked_rem(y) && rem != 0 {
        let quo = x / y;
        (xa, ya) = (ya, xa - quo.cast_signed() * ya);
        (xb, yb) = (yb, xb - quo.cast_signed() * yb);
        (x, y) = (y, rem);
    }
    (y, ya, yb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        939\n\
        7,13,x,x,59,x,31,19\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(result.earliest_departure, 939);
        assert_eq!(
            result.schedule,
            [
                Bus { id: 7, offset: 0 },
                Bus { id: 13, offset: 1 },
                Bus { id: 59, offset: 4 },
                Bus { id: 31, offset: 6 },
                Bus { id: 19, offset: 7 },
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let input = parse(EXAMPLE1).unwrap();
        let result = part_1(&input);
        assert_eq!(result, 295);
    }

    #[test_case(12, 8 => (4, 1, -1))]
    #[test_case(23_894_798_501_898, 23_948_178_468_116 => (2, 2_437_250_447_493, -2_431_817_869_532))]
    fn test_egcd(x: u64, y: u64) -> (u64, i64, i64) {
        extended_gcd(x, y)
    }

    #[test_case(2, 3, 3, 5 => 8)]
    #[test_case(8, 3*5, 2, 7 => 23)]
    fn test_chinese_remainder(a1: u64, n1: u64, a2: u64, n2: u64) -> u64 {
        chinese_remainder(a1, n1, a2, n2)
    }

    #[test_case(EXAMPLE1 => 1_068_781)]
    #[test_case("0\n17,x,13,19" => 3_417)]
    #[test_case("0\n67,7,59,61" => 754_018)]
    #[test_case("0\n67,x,7,59,61" => 779_210)]
    #[test_case("0\n67,7,x,59,61" => 1_261_476)]
    #[test_case("0\n1789,37,47,1889" => 1_202_161_486)]
    fn test_part_2(input: &str) -> u64 {
        let input = parse(input).unwrap();
        part_2(&input)
    }
}
