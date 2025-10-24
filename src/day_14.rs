use std::collections::HashMap;
use std::fmt::{Display, Write};
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
enum Mask {
    Zero = 0,
    One = 1,
    X = 2,
}

impl TryFrom<u8> for Mask {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'0' => Self::Zero,
            b'1' => Self::One,
            b'X' => Self::X,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

impl Display for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Zero => '0',
            Self::One => '1',
            Self::X => 'X',
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Mask([Mask; 36]),
    Memory(u64, u64),
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            if let Some(mask) = s.strip_prefix("mask = ")
                && mask.len() == 36
            {
                let mut arr = [Mask::X; 36];
                for (i, ch) in mask.bytes().enumerate() {
                    arr[i] = ch.try_into()?;
                }
                Self::Mask(arr)
            } else if let Some(rest) = s.strip_prefix("mem[")
                && let Some((addr, value)) = rest.split_once("] = ")
            {
                Self::Memory(addr.parse()?, value.parse()?)
            } else {
                return Err(ParseError::SyntaxError);
            },
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mask(mask) => {
                f.write_str("mask = ")?;
                for &mask_ch in mask {
                    mask_ch.fmt(f)?;
                }
                Ok(())
            }
            &Self::Memory(addr, value) => {
                write!(f, "addr[{addr}] = {value}")
            }
        }
    }
}

#[aoc_generator(day14)]
fn parse(input: &str) -> Result<Vec<Instruction>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day14, part1)]
fn part_1(instructions: &[Instruction]) -> u64 {
    let mut memory = HashMap::new();
    let mut current_mask = [Mask::Zero; 36];
    for instr in instructions {
        match *instr {
            Instruction::Mask(new_mask) => current_mask = new_mask,
            Instruction::Memory(addr, value) => {
                let real_value = apply_mask(value, &current_mask);
                memory.insert(addr, real_value);
            }
        }
    }
    memory.into_values().sum()
}

fn apply_mask(mut addr: u64, mask: &[Mask]) -> u64 {
    for (shift, mask_ch) in mask.iter().rev().enumerate() {
        match mask_ch {
            Mask::Zero => addr &= !(1 << shift),
            Mask::One => addr |= 1 << shift,
            Mask::X => (),
        }
    }
    addr
}

#[aoc(day14, part2)]
fn part_2(instructions: &[Instruction]) -> u64 {
    let mut memory = HashMap::new();
    let mut current_mask = [Mask::Zero; 36];

    for instr in instructions {
        match *instr {
            Instruction::Mask(new_mask) => current_mask = new_mask,
            Instruction::Memory(addr, value) => {
                for real_addr in MaskIterator::new(&current_mask, addr) {
                    memory.insert(real_addr, value);
                }
            }
        }
    }
    memory.into_values().sum()
}

struct MaskIterator<'a> {
    mask: &'a [Mask],
    addr: u64,
    started: bool,
}

impl<'a> MaskIterator<'a> {
    const fn new(mask: &'a [Mask], addr: u64) -> Self {
        Self {
            mask,
            addr,
            started: false,
        }
    }
}

impl Iterator for MaskIterator<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            for (shift, &mask_ch) in self.mask.iter().rev().enumerate() {
                match mask_ch {
                    Mask::One => {
                        self.addr |= 1 << shift;
                    }
                    Mask::X => {
                        self.addr &= !(1 << shift);
                    }
                    Mask::Zero => ()
                }
            }
            self.started = true;
            return Some(self.addr);
        }
        for (shift, &mask_ch) in self.mask.iter().rev().enumerate() {
            if mask_ch == Mask::X {
                if (self.addr & (1 << shift)) == 0 {
                    self.addr |= 1 << shift;
                    return Some(self.addr);
                }
                self.addr &= !(1 << shift);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        mask = XXXXXXXXXXXXXXXXXXXXXXXXXXXXX1XXXX0X\n\
        mem[8] = 11\n\
        mem[7] = 101\n\
        mem[8] = 0\
    ";

    const EXAMPLE2: &str = "\
        mask = 000000000000000000000000000000X1001X\n\
        mem[42] = 100\n\
        mask = 00000000000000000000000000000000X0XX\n\
        mem[26] = 1\
    ";

    #[test]
    fn test_parse() {
        const M0: Mask = Mask::Zero;
        const M1: Mask = Mask::One;
        const MX: Mask = Mask::X;
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result,
            [
                Instruction::Mask([
                    MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX, MX,
                    MX, MX, MX, MX, MX, MX, MX, MX, MX, M1, MX, MX, MX, MX, M0, MX
                ]),
                Instruction::Memory(8, 11),
                Instruction::Memory(7, 101),
                Instruction::Memory(8, 0)
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let instructions = parse(EXAMPLE1).unwrap();
        let result = part_1(&instructions);
        assert_eq!(result, 165);
    }

    #[test]
    fn test_part_2() {
        let instructions = parse(EXAMPLE2).unwrap();
        let result = part_2(&instructions);
        assert_eq!(result, 208);
    }
}
