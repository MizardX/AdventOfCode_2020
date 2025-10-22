use std::collections::VecDeque;
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

type Value = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Acc(Value),
    Nop(Value),
    Jmp(Value),
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (op, arg) = s.split_once(' ').ok_or(ParseError::SyntaxError)?;
        Ok(match op {
            "acc" => Self::Acc(arg.parse()?),
            "nop" => Self::Nop(arg.parse()?),
            "jmp" => Self::Jmp(arg.parse()?),
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[aoc_generator(day8)]
fn parse(input: &str) -> Result<Vec<Instruction>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day8, part1)]
fn part_1(instructions: &[Instruction]) -> Value {
    let mut accumulator = 0;
    let mut ip = 0;
    let mut visited = vec![false; instructions.len()];
    while let Some(&instr) = instructions.get(ip) {
        if visited[ip] {
            return accumulator;
        }
        visited[ip] = true;
        match instr {
            Instruction::Acc(x) => accumulator += x,
            Instruction::Nop(..) => (),
            Instruction::Jmp(x) => {
                ip = ip
                    .checked_add_signed(x as isize)
                    .unwrap_or(instructions.len());
                continue;
            }
        }
        ip += 1;
    }
    accumulator
}

#[aoc(day8, part2)]
fn part_2(instructions: &[Instruction]) -> Value {
    let mut pending = VecDeque::new();
    let n = instructions.len();
    let mut visited = vec![false; n * (n + 1)];
    pending.push_back((0, None, 0));
    while let Some((ip, switched, accum)) = pending.pop_front() {
        if ip >= instructions.len() {
            return accum;
        }
        if visited[switched.unwrap_or(n) * n + ip] {
            continue;
        }
        visited[switched.unwrap_or(n) * n + ip] = true;
        match instructions[ip] {
            Instruction::Acc(x) => {
                pending.push_back((ip + 1, switched, accum + x));
            }
            Instruction::Jmp(x) => {
                let ip2 = ip
                    .checked_add_signed(x as isize)
                    .unwrap_or(instructions.len());
                pending.push_back((ip2, switched, accum));
                if switched.is_none() {
                    pending.push_back((ip + 1, Some(ip), accum));
                }
            }
            Instruction::Nop(x) => {
                pending.push_back((ip + 1, switched, accum));
                if switched.is_none() {
                    let ip2 = ip
                        .checked_add_signed(x as isize)
                        .unwrap_or(instructions.len());
                    pending.push_back((ip2, Some(ip), accum));
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        nop +0\n\
        acc +1\n\
        jmp +4\n\
        acc +3\n\
        jmp -3\n\
        acc -99\n\
        acc +1\n\
        jmp -4\n\
        acc +6\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(
            result,
            [
                Instruction::Nop(0),
                Instruction::Acc(1),
                Instruction::Jmp(4),
                Instruction::Acc(3),
                Instruction::Jmp(-3),
                Instruction::Acc(-99),
                Instruction::Acc(1),
                Instruction::Jmp(-4),
                Instruction::Acc(6),
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let instruction = parse(EXAMPLE).unwrap();
        let result = part_1(&instruction);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_part_2() {
        let instruction = parse(EXAMPLE).unwrap();
        let result = part_2(&instruction);
        assert_eq!(result, 8);
    }
}
