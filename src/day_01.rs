use std::num::ParseIntError;

#[aoc_generator(day1)]
fn parse(input: &str) -> Result<Vec<u16>, ParseIntError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day1, part1)]
fn part_1(expenses: &[u16]) -> u32 {
    let mut seen = vec![false; 2021];
    for &value in expenses {
        if seen[usize::from(2020 - value)] {
            return u32::from(value) * u32::from(2020 - value);
        }
        seen[usize::from(value)] = true;
    }
    0
}

#[aoc(day1, part2)]
fn part_2(expenses: &[u16]) -> u32 {
    let mut seen = vec![None; 2020];
    for (i, &value1) in expenses.iter().enumerate() {
        if let Some(prod) = seen[usize::from(2020 - value1)] {
            return prod * u32::from(value1);
        }
        for &value2 in &expenses[..i] {
            if value1 + value2 < 2020 {
                let prod = u32::from(value1) * u32::from(value2);
                seen[usize::from(value1 + value2)] = Some(prod);
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        1721\n\
        979\n\
        366\n\
        299\n\
        675\n\
        1456\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(result, [1721, 979, 366, 299, 675, 1456]);
    }

    #[test]
    fn test_part_1() {
        let expenses = parse(EXAMPLE).unwrap();
        let result = part_1(&expenses);
        assert_eq!(result, 514_579);
    }

    #[test]
    fn test_part_2() {
        let expenses = parse(EXAMPLE).unwrap();
        let result = part_2(&expenses);
        assert_eq!(result, 241_861_950);
    }
}
