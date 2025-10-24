use std::collections::HashMap;
use std::num::{NonZeroUsize, ParseIntError};

#[aoc_generator(day15)]
fn parse(input: &str) -> Result<Vec<u32>, ParseIntError> {
    input.split(',').map(str::parse).collect()
}

#[aoc(day15, part1)]
fn part_1(input: &[u32]) -> u32 {
    NumberSpeaker::new(input).nth(2020 - 1).unwrap()
}

#[aoc(day15, part2)]
fn part_2(input: &[u32]) -> u32 {
    NumberSpeaker::new(input).nth(30_000_000 - 1).unwrap()
}

struct NumberSpeaker {
    initial_numbers: Vec<u32>,
    turn: usize,
    prev_number: Option<u32>,
    history_low: Vec<Option<NonZeroUsize>>,
    history_high: HashMap<u32, usize>,
}

const LIMIT: u32 = 1_000_000;

impl NumberSpeaker {
    fn new(initial_numbers: &[u32]) -> Self {
        Self {
            initial_numbers: initial_numbers.to_vec(),
            turn: 1,
            prev_number: None,
            history_low: vec![None; LIMIT as usize],
            history_high: HashMap::new(),
        }
    }

    fn get_history(&self, value: u32) -> Option<usize> {
        if value < LIMIT {
            self.history_low[value as usize].map(NonZeroUsize::get)
        } else {
            self.history_high.get(&value).copied()
        }
    }

    fn set_history(&mut self, value: u32, round: usize) {
        if value < LIMIT {
            self.history_low[value as usize] = NonZeroUsize::new(round);
        } else {
            self.history_high.insert(value, round);
        }
    }
}

impl Iterator for NumberSpeaker {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let next_number = if self.turn <= self.initial_numbers.len() {
            self.initial_numbers[self.turn - 1]
        } else if let Some(last_round) = self.get_history(self.prev_number.unwrap()) {
            u32::try_from(self.turn - last_round - 1).unwrap()
        } else {
            0
        };
        if let Some(prev_number) = self.prev_number {
            self.set_history(prev_number, self.turn - 1);
        }
        self.turn += 1;
        self.prev_number = Some(next_number);
        Some(next_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("0,3,6" => 436)]
    #[test_case("1,3,2" => 1)]
    #[test_case("2,1,3" => 10)]
    #[test_case("1,2,3" => 27)]
    #[test_case("2,3,1" => 78)]
    #[test_case("3,2,1" => 438)]
    #[test_case("3,1,2" => 1836)]
    fn test_part_1(input: &str) -> u32 {
        let nums = parse(input).unwrap();
        part_1(&nums)
    }

    #[ignore = "slow"]
    #[test_case("0,3,6" => 175_594)]
    #[test_case("1,3,2" => 2_578)]
    #[test_case("2,1,3" => 3_544_142)]
    #[test_case("1,2,3" => 261_214)]
    #[test_case("2,3,1" => 6_895_259)]
    #[test_case("3,2,1" => 18)]
    #[test_case("3,1,2" => 362)]
    fn test_part_2(input: &str) -> u32 {
        let nums = parse(input).unwrap();
        part_2(&nums)
    }
}
