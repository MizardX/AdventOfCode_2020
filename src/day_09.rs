use std::cmp::Ordering;
use std::collections::HashMap;
use std::num::ParseIntError;

#[aoc_generator(day9)]
fn parse(input: &str) -> Result<Vec<u64>, ParseIntError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day9, part1)]
fn part_1(xmas: &[u64]) -> u64 {
    find_invalid_number(xmas, 25)
}

fn find_invalid_number(xmas: &[u64], preamble: usize) -> u64 {
    let mut counts = HashMap::<u64, u8>::new();
    for (ix, &x) in xmas[..preamble].iter().enumerate() {
        for &y in &xmas[..ix] {
            *counts.entry(x + y).or_default() += 1;
        }
    }
    for (window, &add) in xmas.windows(preamble).zip(&xmas[preamble..]) {
        if counts.get(&add).is_none_or(|&cnt| cnt == 0) {
            return add;
        }
        let x = window[0];
        for &y in &window[1..] {
            counts.entry(x + y).and_modify(|count| *count -= 1);
            *counts.entry(add + y).or_default() += 1;
        }
    }
    0
}

#[aoc(day9, part2)]
fn part_2(xmas: &[u64]) -> u64 {
    let invalid = find_invalid_number(xmas, 25);
    find_subsequence_with_sum(xmas, invalid)
}

fn find_subsequence_with_sum(xmas: &[u64], target_sum: u64) -> u64 {
    let mut left = 0;
    let mut right = 0;
    let mut sum = 0;
    loop {
        match sum.cmp(&target_sum) {
            Ordering::Less => {
                if right >= xmas.len() {
                    // Could not find a subsequence that adds to `target_sum`
                    return 0;
                }
                sum += xmas[right];
                right += 1;
            }
            Ordering::Greater => {
                sum -= xmas[left];
                left += 1;
            }
            Ordering::Equal => {
                return xmas[left..right].iter().copied().min().unwrap()
                    + xmas[left..right].iter().copied().max().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        35\n\
        20\n\
        15\n\
        25\n\
        47\n\
        40\n\
        62\n\
        55\n\
        65\n\
        95\n\
        102\n\
        117\n\
        150\n\
        182\n\
        127\n\
        219\n\
        299\n\
        277\n\
        309\n\
        576\
    ";

    #[test]
    fn test_find_invalid_number() {
        let xmas = parse(EXAMPLE).unwrap();
        let result = find_invalid_number(&xmas, 5);
        assert_eq!(result, 127);
    }
    #[test]
    fn test_find_subsequence_with_sum() {
        let xmas = parse(EXAMPLE).unwrap();
        let result = find_subsequence_with_sum(&xmas, 127);
        assert_eq!(result, 62);
    }
}
