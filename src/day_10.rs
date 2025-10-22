use std::num::ParseIntError;

#[aoc_generator(day10)]
fn parse(input: &str) -> Result<Vec<u32>, ParseIntError> {
    let mut result = input
        .lines()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;
    result.push(0);
    result.push(result.iter().copied().max().unwrap() + 3);
    result.sort_unstable();
    Ok(result)
}

#[aoc(day10, part1)]
fn part_1(voltage: &[u32]) -> u32 {
    let mut diff_counts = [0; 4];
    for (&v1, &v2) in voltage.iter().zip(&voltage[1..]) {
        let dv = v2 - v1;
        diff_counts[dv as usize] += 1;
    }

    println!("{diff_counts:?}");

    diff_counts[1] * diff_counts[3]
}

#[aoc(day10, part2)]
fn part_2(voltage: &[u32]) -> u64 {
    let mut dp = vec![0; voltage.len()];
    dp[0] = 1;
    let mut left = 0;
    for (right, &v) in voltage.iter().enumerate().skip(1) {
        while voltage[left] + 3 < v {
            left += 1;
        }
        dp[right] = dp[left..right].iter().copied().sum();
    }
    dp.last().copied().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        16\n\
        10\n\
        15\n\
        5\n\
        1\n\
        11\n\
        7\n\
        19\n\
        6\n\
        12\n\
        4\
    ";

    const EXAMPLE2: &str = "\
        28\n\
        33\n\
        18\n\
        42\n\
        31\n\
        14\n\
        46\n\
        20\n\
        48\n\
        47\n\
        24\n\
        23\n\
        49\n\
        45\n\
        19\n\
        38\n\
        39\n\
        11\n\
        1\n\
        32\n\
        25\n\
        35\n\
        8\n\
        17\n\
        7\n\
        9\n\
        4\n\
        2\n\
        34\n\
        10\n\
        3\
    ";

    #[test_case(EXAMPLE1 => 35)]
    #[test_case(EXAMPLE2 => 220)]
    fn test_part_1(input: &str) -> u32 {
        let voltages = parse(input).unwrap();
        part_1(&voltages)
    }

    #[test_case(EXAMPLE1 => 8)]
    #[test_case(EXAMPLE2 => 19208)]
    fn test_part_2(input: &str) -> u64 {
        let voltages = parse(input).unwrap();
        part_2(&voltages)
    }
}
