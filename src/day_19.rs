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
enum Alt {
    None,
    Only(Seq),
    Either(Seq, Seq),
}

impl FromStr for Alt {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((left, right)) = s.split_once(" | ") {
            Self::Either(left.parse()?, right.parse()?)
        } else {
            Self::Only(s.parse()?)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seq {
    Only(Leaf),
    Both(Leaf, Leaf),
    AllThree(Leaf, Leaf, Leaf),
}

impl FromStr for Seq {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((first, rest)) = s.split_once(' ') {
            if let Some((second, third)) = rest.split_once(' ') {
                Self::AllThree(first.parse()?, second.parse()?, third.parse()?)
            } else {
                Self::Both(first.parse()?, rest.parse()?)
            }
        } else {
            Self::Only(s.parse()?)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leaf {
    Lit(u8),
    Rule(usize),
}

impl FromStr for Leaf {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let &[b'"', ch, b'"'] = s.as_bytes() {
            Self::Lit(ch)
        } else {
            Self::Rule(s.parse()?)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rule {
    id: usize,
    matches: Alt,
}

impl FromStr for Rule {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id, text) = s.split_once(": ").ok_or(ParseError::SyntaxError)?;
        Ok(Self {
            id: id.parse()?,
            matches: text.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
struct Input {
    rules: Vec<Rule>,
    messages: Vec<Vec<u8>>,
}

impl FromStr for Input {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut unordered_rules = Vec::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            unordered_rules.push(line.parse::<Rule>()?);
        }
        let max_id = unordered_rules.iter().map(|r| r.id).max().unwrap();
        let mut rules = (0..=max_id)
            .map(|id| Rule {
                id,
                matches: Alt::None,
            })
            .collect::<Vec<_>>();
        for r in unordered_rules {
            rules[r.id] = r;
        }

        let mut messages = Vec::new();
        for line in lines {
            messages.push(line.as_bytes().to_vec());
        }
        Ok(Self { rules, messages })
    }
}

#[aoc_generator(day19)]
fn parse(input: &str) -> Result<Input, ParseError> {
    input.parse()
}

#[aoc(day19, part1)]
fn part_1(input: &Input) -> usize {
    let validator = MessageValidator::new(&input.rules);
    input
        .messages
        .iter()
        .filter(|msg| validator.validate(msg))
        .count()
}

#[aoc(day19, part2)]
fn part_2(input: &Input) -> usize {
    let mut rules = input.rules.clone();
    rules[8].matches = Alt::Either(
        Seq::Only(Leaf::Rule(42)),
        Seq::Both(Leaf::Rule(42), Leaf::Rule(8)),
    );
    rules[11].matches = Alt::Either(
        Seq::Both(Leaf::Rule(42), Leaf::Rule(31)),
        Seq::AllThree(Leaf::Rule(42), Leaf::Rule(11), Leaf::Rule(31)),
    );
    let validator = MessageValidator::new(&rules);
    input
        .messages
        .iter()
        .filter(|msg| validator.validate(msg))
        .count()
}

#[derive(Debug)]
struct MessageValidator<'a> {
    rules: &'a [Rule],
}

impl<'a> MessageValidator<'a> {
    const fn new(rules: &'a [Rule]) -> Self {
        Self { rules }
    }
}

impl MessageValidator<'_> {
    fn validate(&self, message: &[u8]) -> bool {
        self.validate_rule(0, message, &mut |msg| msg.is_empty())
    }

    fn validate_rule(
        &self,
        rule_id: usize,
        message: &[u8],
        continue_with: &mut dyn FnMut(&[u8]) -> bool,
    ) -> bool {
        self.validate_alt(self.rules[rule_id].matches, message, continue_with)
    }

    fn validate_alt(
        &self,
        alt: Alt,
        message: &[u8],
        continue_with: &mut dyn FnMut(&[u8]) -> bool,
    ) -> bool {
        match alt {
            Alt::None => panic!("Tried to match empty rule"),
            Alt::Only(seq) => self.validate_seq(seq, message, continue_with),
            Alt::Either(seq1, seq2) => {
                self.validate_seq(seq1, message, continue_with)
                    || self.validate_seq(seq2, message, continue_with)
            }
        }
    }

    fn validate_seq(
        &self,
        seq: Seq,
        message: &[u8],
        continue_with: &mut dyn FnMut(&[u8]) -> bool,
    ) -> bool {
        match seq {
            Seq::Only(leaf) => self.validate_leaf(leaf, message, continue_with),
            Seq::Both(leaf1, leaf2) => self.validate_leaf(leaf1, message, &mut |message2| {
                self.validate_leaf(leaf2, message2, continue_with)
            }),
            Seq::AllThree(leaf1, leaf2, leaf3) => {
                self.validate_leaf(leaf1, message, &mut |message2| {
                    self.validate_leaf(leaf2, message2, &mut |message3| {
                        self.validate_leaf(leaf3, message3, continue_with)
                    })
                })
            }
        }
    }

    fn validate_leaf(
        &self,
        leaf: Leaf,
        message: &[u8],
        continue_with: &mut dyn FnMut(&[u8]) -> bool,
    ) -> bool {
        match leaf {
            Leaf::Lit(ch) => message.starts_with(&[ch]) && continue_with(&message[1..]),
            Leaf::Rule(id) => self.validate_rule(id, message, continue_with),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        0: 4 1 5\n\
        1: 2 3 | 3 2\n\
        2: 4 4 | 5 5\n\
        3: 4 5 | 5 4\n\
        4: \"a\"\n\
        5: \"b\"\n\
        \n\
        ababbb\n\
        bababa\n\
        abbbab\n\
        aaabbb\n\
        aaaabbb\
    ";

    const EXAMPLE2: &str = "\
        42: 9 14 | 10 1\n\
        9: 14 27 | 1 26\n\
        10: 23 14 | 28 1\n\
        1: \"a\"\n\
        11: 42 31\n\
        5: 1 14 | 15 1\n\
        19: 14 1 | 14 14\n\
        12: 24 14 | 19 1\n\
        16: 15 1 | 14 14\n\
        31: 14 17 | 1 13\n\
        6: 14 14 | 1 14\n\
        2: 1 24 | 14 4\n\
        0: 8 11\n\
        13: 14 3 | 1 12\n\
        15: 1 | 14\n\
        17: 14 2 | 1 7\n\
        23: 25 1 | 22 14\n\
        28: 16 1\n\
        4: 1 1\n\
        20: 14 14 | 1 15\n\
        3: 5 14 | 16 1\n\
        27: 1 6 | 14 18\n\
        14: \"b\"\n\
        21: 14 1 | 1 14\n\
        25: 1 1 | 1 14\n\
        22: 14 14\n\
        8: 42\n\
        26: 14 22 | 1 20\n\
        18: 15 15\n\
        7: 14 5 | 1 21\n\
        24: 14 1\n\
        \n\
        abbbbbabbbaaaababbaabbbbabababbbabbbbbbabaaaa\n\
        bbabbbbaabaabba\n\
        babbbbaabbbbbabbbbbbaabaaabaaa\n\
        aaabbbbbbaaaabaababaabababbabaaabbababababaaa\n\
        bbbbbbbaaaabbbbaaabbabaaa\n\
        bbbababbbbaaaaaaaabbababaaababaabab\n\
        ababaaaaaabaaab\n\
        ababaaaaabbbaba\n\
        baabbaaaabbaaaababbaababb\n\
        abbbbabbbbaaaababbbbbbaaaababb\n\
        aaaaabbaabaaaaababaa\n\
        aaaabbaaaabbaaa\n\
        aaaabbaabbaaaaaaabbbabbbaaabbaabaaa\n\
        babaaabbbaaabaababbaabababaaab\n\
        aabbbbbaabbbaaaaaabbbbbababaaaaabbaaabba\
    ";

    #[test]
    fn test_part_1() {
        let input = parse(EXAMPLE1).unwrap();
        let result = part_1(&input);
        assert_eq!(result, 2);
    }

    #[test]
    fn test_part_2() {
        let input = parse(EXAMPLE2).unwrap();
        let result = part_2(&input);
        assert_eq!(result, 12);
    }
}
