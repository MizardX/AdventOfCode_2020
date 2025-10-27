use std::collections::{HashSet, VecDeque};
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
enum Player {
    Player1,
    Player2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
struct State {
    player1: VecDeque<u16>,
    player2: VecDeque<u16>,
}

impl State {
    fn deal(&mut self) -> Option<(u16, u16)> {
        if self.player2.is_empty() {
            None
        } else {
            Some((self.player1.pop_front()?, self.player2.pop_front().unwrap()))
        }
    }

    fn keep_both(&mut self, player: Player, card1: u16, card2: u16) {
        match player {
            Player::Player1 => {
                self.player1.push_back(card1);
                self.player1.push_back(card2);
            }
            Player::Player2 => {
                self.player2.push_back(card2);
                self.player2.push_back(card1);
            }
        }
    }

    fn play_simple_game(&mut self) -> Player {
        while let Some((card1, card2)) = self.deal() {
            let winner = if card1 > card2 {
                Player::Player1
            } else {
                Player::Player2
            };
            self.keep_both(winner, card1, card2);
        }
        if self.player2.is_empty() {
            Player::Player1
        } else {
            Player::Player2
        }
    }

    fn make_subgame(&self, size1: u16, size2: u16) -> Option<Self> {
        let size1 = usize::from(size1);
        let size2 = usize::from(size2);
        if size1 > self.player1.len() || size2 > self.player2.len() {
            return None;
        }

        let mut new_game = Self::default();
        new_game.player1.reserve(size1 + size2);
        new_game.player2.reserve(size2 + size2);

        let (xs1, ys1) = self.player1.as_slices();
        let (xs2, ys2) = self.player2.as_slices();
        if size1 <= xs1.len() {
            new_game.player1.extend(&xs1[..size1]);
        } else {
            new_game.player1.extend(xs1);
            new_game.player1.extend(&ys1[..size1 - xs1.len()]);
        }
        if size2 <= xs2.len() {
            new_game.player2.extend(&xs2[..size2]);
        } else {
            new_game.player2.extend(xs2);
            new_game.player2.extend(&ys2[..size2 - xs2.len()]);
        }
        Some(new_game)
    }

    fn play_recursive_game(&mut self) -> Player {
        let mut seen_rounds = HashSet::new();
        while let Some((card1, card2)) = self.deal() {
            #[expect(clippy::option_if_let_else, reason = "Readablility")]
            let winner = if let Some(mut subgame) = self.make_subgame(card1, card2) {
                subgame.play_recursive_game()
            } else if card1 > card2 {
                Player::Player1
            } else {
                Player::Player2
            };
            self.keep_both(winner, card1, card2);
            if seen_rounds.contains(self) {
                return Player::Player1;
            }
            seen_rounds.insert(self.clone());
        }
        if self.player2.is_empty() {
            Player::Player1
        } else {
            Player::Player2
        }
    }

    fn calculate_score(&self, player: Player) -> u64 {
        let hand = match player {
            Player::Player1 => &self.player1,
            Player::Player2 => &self.player2,
        };
        hand.iter()
            .rev()
            .zip(1..)
            .map(|(&card, mult)| u64::from(card) * mult)
            .sum()
    }
}

impl FromStr for State {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        if lines.next() != Some("Player 1:") {
            return Err(ParseError::SyntaxError);
        }
        let mut player1 = VecDeque::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break;
            }
            player1.push_back(line.parse()?);
        }
        if lines.next() != Some("Player 2:") {
            return Err(ParseError::SyntaxError);
        }
        let mut player2 = VecDeque::new();
        for line in lines {
            player2.push_back(line.parse()?);
        }
        Ok(Self {
            player1,
            player2,
        })
    }
}

#[aoc_generator(day22)]
fn parse(input: &str) -> Result<State, ParseError> {
    input.parse()
}

#[aoc(day22, part1)]
fn part_1(initial_state: &State) -> u64 {
    let mut game = initial_state.clone();
    let winner = game.play_simple_game();
    game.calculate_score(winner)
}

#[aoc(day22, part2)]
fn part_2(initial_state: &State) -> u64 {
    let mut game = initial_state.clone();

    let winner = game.play_recursive_game();
    game.calculate_score(winner)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        Player 1:\n\
        9\n\
        2\n\
        6\n\
        3\n\
        1\n\
        \n\
        Player 2:\n\
        5\n\
        8\n\
        4\n\
        7\n\
        10\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(result.player1, [9, 2, 6, 3, 1]);
        assert_eq!(result.player2, [5, 8, 4, 7, 10]);
    }

    #[test]
    fn test_part_1() {
        let initial_state = parse(EXAMPLE).unwrap();
        let result = part_1(&initial_state);
        assert_eq!(result, 306);
    }

    #[test]
    fn test_part_2() {
        let initial_state = parse(EXAMPLE).unwrap();
        let result = part_2(&initial_state);
        assert_eq!(result, 291);
    }
}
