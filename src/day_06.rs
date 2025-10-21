use std::ops::{BitAnd, BitOr};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Group {
    answers: Vec<Answers>,
}

impl FromStr for Group {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            answers: s
                .lines()
                .map(str::parse)
                .collect::<Result<Vec<_>, ParseError>>()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Answers(u32);

impl Answers {
    const fn count_ones(self) -> u32 {
        self.0.count_ones()
    }
}

impl FromStr for Answers {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mask = 0;
        for ch in s.bytes() {
            if ch.is_ascii_lowercase() {
                mask |= 1 << (ch - b'a');
            } else {
                return Err(ParseError::SyntaxError);
            }
        }
        Ok(Self(mask))
    }
}

impl BitOr for Answers {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.0 |= rhs.0;
        self
    }
}

impl BitAnd for Answers {
    type Output = Self;

    fn bitand(mut self, rhs: Self) -> Self::Output {
        self.0 &= rhs.0;
        self
    }
}

#[aoc_generator(day6)]
fn parse(input: &str) -> Result<Vec<Group>, ParseError> {
    input.split("\n\n").map(str::parse).collect()
}

#[aoc(day6, part1)]
fn part_1(groups: &[Group]) -> u32 {
    groups
        .iter()
        .map(|g| {
            g.answers
                .iter()
                .copied()
                .reduce(BitOr::bitor)
                .unwrap()
                .count_ones()
        })
        .sum()
}

#[aoc(day6, part2)]
fn part_2(groups: &[Group]) -> u32 {
    groups
        .iter()
        .map(|g| {
            g.answers
                .iter()
                .copied()
                .reduce(BitAnd::bitand)
                .unwrap()
                .count_ones()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        abc\n\
        \n\
        a\n\
        b\n\
        c\n\
        \n\
        ab\n\
        ac\n\
        \n\
        a\n\
        a\n\
        a\n\
        a\n\
        \n\
        b\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(
            result,
            [
                Group {
                    answers: vec![Answers(0b111)]
                },
                Group {
                    answers: vec![Answers(0b1), Answers(0b10), Answers(0b100)]
                },
                Group {
                    answers: vec![Answers(0b11), Answers(0b101)]
                },
                Group {
                    answers: vec![Answers(0b1), Answers(0b1), Answers(0b1), Answers(0b1)]
                },
                Group {
                    answers: vec![Answers(0b10)]
                },
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let gropus = parse(EXAMPLE).unwrap();
        let result = part_1(&gropus);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_part_2() {
        let gropus = parse(EXAMPLE).unwrap();
        let result = part_2(&gropus);
        assert_eq!(result, 6);
    }
}
