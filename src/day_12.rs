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
enum Rotation {
    Right,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Degrees {
    Deg90,
    Deg180,
    Deg270,
}

impl FromStr for Degrees {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "90" => Self::Deg90,
            "180" => Self::Deg180,
            "270" => Self::Deg270,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cardinal {
    North,
    East,
    South,
    West,
}

impl Cardinal {
    const fn rotate(&mut self, rot: Rotation, deg: Degrees) {
        *self = match (*self, (rot, deg)) {
            (Self::East, (Rotation::Left, Degrees::Deg90) | (Rotation::Right, Degrees::Deg270))
            | (Self::South, (_, Degrees::Deg180))
            | (Self::West, (Rotation::Left, Degrees::Deg270) | (Rotation::Right, Degrees::Deg90)) => {
                Self::North
            }
            (
                Self::North,
                (Rotation::Left, Degrees::Deg270) | (Rotation::Right, Degrees::Deg90),
            )
            | (
                Self::South,
                (Rotation::Left, Degrees::Deg90) | (Rotation::Right, Degrees::Deg270),
            )
            | (Self::West, (_, Degrees::Deg180)) => Self::East,
            (Self::North, (_, Degrees::Deg180))
            | (Self::East, (Rotation::Left, Degrees::Deg270) | (Rotation::Right, Degrees::Deg90))
            | (Self::West, (Rotation::Left, Degrees::Deg90) | (Rotation::Right, Degrees::Deg270)) => {
                Self::South
            }
            (
                Self::North,
                (Rotation::Left, Degrees::Deg90) | (Rotation::Right, Degrees::Deg270),
            )
            | (Self::East, (_, Degrees::Deg180))
            | (
                Self::South,
                (Rotation::Left, Degrees::Deg270) | (Rotation::Right, Degrees::Deg90),
            ) => Self::West,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Cardinal(Cardinal, u8),
    Rotation(Rotation, Degrees),
    Forward(u8),
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match *s.as_bytes() {
            [b'N', ..] => Self::Cardinal(Cardinal::North, s[1..].parse()?),
            [b'E', ..] => Self::Cardinal(Cardinal::East, s[1..].parse()?),
            [b'S', ..] => Self::Cardinal(Cardinal::South, s[1..].parse()?),
            [b'W', ..] => Self::Cardinal(Cardinal::West, s[1..].parse()?),
            [b'L', ..] => Self::Rotation(Rotation::Left, s[1..].parse()?),
            [b'R', ..] => Self::Rotation(Rotation::Right, s[1..].parse()?),
            [b'F', ..] => Self::Forward(s[1..].parse()?),
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Position {
    north: i64,
    east: i64,
}

impl Position {
    const fn new(north: i64, east: i64) -> Self {
        Self { north, east }
    }

    fn move_cardinal(&mut self, direction: Cardinal, dist: u8) {
        match direction {
            Cardinal::North => self.north += i64::from(dist),
            Cardinal::East => self.east += i64::from(dist),
            Cardinal::South => self.north -= i64::from(dist),
            Cardinal::West => self.east -= i64::from(dist),
        }
    }

    const fn dist(self) -> u64 {
        self.east.unsigned_abs() + self.north.unsigned_abs()
    }

    fn move_by_waypoint(&mut self, waypoint: Self, times: u8) {
        self.east += waypoint.east * i64::from(times);
        self.north += waypoint.north * i64::from(times);
    }

    const fn rotate(&mut self, rot: Rotation, deg: Degrees) {
        (self.east, self.north) = match (rot, deg) {
            (Rotation::Right, Degrees::Deg90) | (Rotation::Left, Degrees::Deg270) => {
                (self.north, -self.east)
            }
            (_, Degrees::Deg180) => (-self.east, -self.north),
            (Rotation::Right, Degrees::Deg270) | (Rotation::Left, Degrees::Deg90) => {
                (-self.north, self.east)
            }
        }
    }
}

#[aoc_generator(day12)]
fn parse(input: &str) -> Result<Vec<Instruction>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day12, part1)]
fn part_1(instructions: &[Instruction]) -> u64 {
    let mut ship = Position::default();
    let mut facing = Cardinal::East;
    for &instr in instructions {
        match instr {
            Instruction::Cardinal(dir, dist) => ship.move_cardinal(dir, dist),
            Instruction::Rotation(rot, deg) => facing.rotate(rot, deg),
            Instruction::Forward(dist) => ship.move_cardinal(facing, dist),
        }
    }
    ship.dist()
}

#[aoc(day12, part2)]
fn part_2(instructions: &[Instruction]) -> u64 {
    let mut ship = Position::default();
    let mut waypoint: Position = Position::new(1, 10);
    for &instr in instructions {
        match instr {
            Instruction::Cardinal(dir, dist) => waypoint.move_cardinal(dir, dist),
            Instruction::Rotation(rot, deg) => waypoint.rotate(rot, deg),
            Instruction::Forward(times) => ship.move_by_waypoint(waypoint, times),
        }
    }
    ship.dist()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        F10\n\
        N3\n\
        F7\n\
        R90\n\
        F11\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(
            result,
            [
                Instruction::Forward(10),
                Instruction::Cardinal(Cardinal::North, 3),
                Instruction::Forward(7),
                Instruction::Rotation(Rotation::Right, Degrees::Deg90),
                Instruction::Forward(11),
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let instructions = parse(EXAMPLE).unwrap();
        let result = part_1(&instructions);
        assert_eq!(result, 25);
    }

    #[test]
    fn test_part_2() {
        let instructions = parse(EXAMPLE).unwrap();
        let result = part_2(&instructions);
        assert_eq!(result, 286);
    }
}
