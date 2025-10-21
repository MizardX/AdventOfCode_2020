use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FrontBack {
    #[default]
    Front = 0,
    Back = 1,
}

impl TryFrom<u8> for FrontBack {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'F' => Self::Front,
            b'B' => Self::Back,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum LeftRight {
    #[default]
    Left = 0,
    Right = 1,
}

impl TryFrom<u8> for LeftRight {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'L' => Self::Left,
            b'R' => Self::Right,
            _ => return Err(ParseError::SyntaxError),
        })
    }
}

const FRONT_BACK_LEN: usize = 7;
const LEFT_RIGHT_LEN: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct SeatId {
    front_back: [FrontBack; FRONT_BACK_LEN],
    left_right: [LeftRight; LEFT_RIGHT_LEN],
}

impl SeatId {
    fn id(&self) -> usize {
        let mut id = 0;
        for fb in self.front_back {
            id = (id << 1) + fb as usize;
        }
        for lr in self.left_right {
            id = (id << 1) + lr as usize;
        }
        id
    }
}

impl FromStr for SeatId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != FRONT_BACK_LEN + LEFT_RIGHT_LEN {
            return Err(ParseError::SyntaxError);
        }
        let mut res = Self::default();
        for (i, ch) in s[..FRONT_BACK_LEN].bytes().enumerate() {
            res.front_back[i] = ch.try_into()?;
        }
        for (i, ch) in s[FRONT_BACK_LEN..].bytes().enumerate() {
            res.left_right[i] = ch.try_into()?;
        }
        Ok(res)
    }
}

#[aoc_generator(day5)]
fn parse(input: &str) -> Result<Vec<SeatId>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day5, part1)]
fn part_1(seat_ids: &[SeatId]) -> usize {
    seat_ids.iter().map(SeatId::id).max().unwrap()
}

#[aoc(day5, part2)]
fn part_2(seat_ids: &[SeatId]) -> usize {
    let mut seats = [false; 8192];
    for id in seat_ids {
        seats[id.id()] = true;
    }
    seats
        .into_iter()
        .enumerate()
        .skip_while(|&(_, found)| !found)
        .find_map(|(id, found)| (!found).then_some(id))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("BFFFBBFRRR" => 567)]
    #[test_case("FFFBBBFRRR" => 119)]
    #[test_case("BBFFBBFRLL" => 820)]
    fn test_seat_id(input: &str) -> usize {
        SeatId::from_str(input).unwrap().id()
    }
}
