use std::num::ParseIntError;

use thiserror::Error;

#[aoc(day2, part1)]
fn part_1(entries: &str) -> usize {
    entries
        .lines()
        .map(|line| PasswordEntry::try_from(line).unwrap())
        .filter(PasswordEntry::is_valid_old)
        .count()
}

#[aoc(day2, part2)]
fn part_2(entries: &str) -> usize {
    entries
        .lines()
        .map(|line| PasswordEntry::try_from(line).unwrap())
        .filter(PasswordEntry::is_valid_new)
        .count()
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PasswordEntry<'a> {
    low: u8,
    high: u8,
    letter: u8,
    password: &'a [u8],
}

impl PasswordEntry<'_> {
    fn is_valid_old(&self) -> bool {
        let mut count = 0;
        for &ch in self.password {
            if ch == self.letter {
                count += 1;
            }
        }
        (self.low..=self.high).contains(&count)
    }

    fn is_valid_new(&self) -> bool {
        let first = self
            .password
            .get(usize::from(self.low - 1))
            .is_some_and(|&ch| ch == self.letter);
        let second = self
            .password
            .get(usize::from(self.high - 1))
            .is_some_and(|&ch| ch == self.letter);
        first ^ second
    }
}

impl<'a> TryFrom<&'a str> for PasswordEntry<'a> {
    type Error = ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let (low, rest) = value.split_once('-').ok_or(ParseError::SyntaxError)?;
        let low: u8 = low.parse()?;
        let (high, rest) = rest.split_once(' ').ok_or(ParseError::SyntaxError)?;
        let high: u8 = high.parse()?;
        let (letter, password) = rest.split_once(": ").ok_or(ParseError::SyntaxError)?;
        let &[letter] = letter.as_bytes() else {
            return Err(ParseError::SyntaxError);
        };
        let password = password.as_bytes();
        Ok(Self {
            low,
            high,
            letter,
            password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        1-3 a: abcde\n\
        1-3 b: cdefg\n\
        2-9 c: ccccccccc\
    ";

    #[test]
    fn test_parse() {
        let result = EXAMPLE
            .lines()
            .map(PasswordEntry::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            result,
            [
                PasswordEntry {
                    low: 1,
                    high: 3,
                    letter: b'a',
                    password: b"abcde",
                },
                PasswordEntry {
                    low: 1,
                    high: 3,
                    letter: b'b',
                    password: b"cdefg",
                },
                PasswordEntry {
                    low: 2,
                    high: 9,
                    letter: b'c',
                    password: b"ccccccccc",
                },
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let result = part_1(EXAMPLE);
        assert_eq!(result, 2);
    }

    #[test]
    fn test_part_2() {
        let result = part_2(EXAMPLE);
        assert_eq!(result, 1);
    }
}
