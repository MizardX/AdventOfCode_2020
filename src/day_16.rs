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
enum Category {
    Departure,
    Other,
}

impl FromStr for Category {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.starts_with("departure ") {
            Self::Departure
        } else {
            Self::Other
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ValueRange {
    low: u16,
    high: u16,
}

impl ValueRange {
    fn try_merge(self, other: Self) -> Option<Self> {
        (self.high + 1 >= other.low && other.high + 1 >= self.low).then_some(Self {
            low: self.low.min(other.low),
            high: self.high.max(other.high),
        })
    }

    const fn contains(self, value: u16) -> bool {
        self.low <= value && value <= self.high
    }

    fn collapse_overlapping_ranges(ranges: &mut Vec<Self>) {
        ranges.sort_unstable();
        let mut prev_opt = None::<Self>;
        ranges.retain_mut(|rng| {
            if let Some(prev) = prev_opt {
                if let Some(merged) = prev.try_merge(*rng) {
                    prev_opt = Some(merged);
                    false
                } else {
                    (prev_opt, *rng) = (Some(*rng), prev);
                    true
                }
            } else {
                prev_opt = Some(*rng);
                false
            }
        });
        if let Some(prev_inner) = prev_opt {
            ranges.push(prev_inner);
        }
    }
}

impl FromStr for ValueRange {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (low, high) = s.split_once('-').ok_or(ParseError::SyntaxError)?;
        Ok(Self {
            low: low.parse()?,
            high: high.parse()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Field {
    category: Category,
    valid_ranges: [ValueRange; 2],
}

impl FromStr for Field {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (label, rest) = s.split_once(": ").ok_or(ParseError::SyntaxError)?;
        let (first, second) = rest.split_once(" or ").ok_or(ParseError::SyntaxError)?;
        Ok(Self {
            category: label.parse()?,
            valid_ranges: [first.parse()?, second.parse()?],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Ticket {
    values: Vec<u16>,
}

impl Ticket {
    fn is_valid(&self, ranges: &[ValueRange]) -> bool {
        self.values
            .iter()
            .all(|&val| ranges.iter().any(|r| r.contains(val)))
    }
}

impl FromStr for Ticket {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            values: s.split(',').map(str::parse).collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Input {
    fields: Vec<Field>,
    your_ticket: Ticket,
    nearby_tickets: Vec<Ticket>,
}

impl FromStr for Input {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let mut fields = Vec::new();
        while let Some(line) = lines.next()
            && !line.is_empty()
        {
            fields.push(line.parse()?);
        }
        if lines.next().ok_or(ParseError::SyntaxError)? != "your ticket:" {
            return Err(ParseError::SyntaxError);
        }
        let your_ticket = lines.next().ok_or(ParseError::SyntaxError)?.parse()?;
        if !lines.next().ok_or(ParseError::SyntaxError)?.is_empty() {
            return Err(ParseError::SyntaxError);
        }
        if lines.next().ok_or(ParseError::SyntaxError)? != "nearby tickets:" {
            return Err(ParseError::SyntaxError);
        }
        let nearby_tickets = lines.map(str::parse).collect::<Result<_, _>>()?;

        Ok(Self {
            fields,
            your_ticket,
            nearby_tickets,
        })
    }
}

#[aoc_generator(day16)]
fn parse(input: &str) -> Result<Input, ParseError> {
    input.parse()
}

#[aoc(day16, part1)]
fn part_1(input: &Input) -> u32 {
    let mut ranges = input
        .fields
        .iter()
        .flat_map(|f| f.valid_ranges)
        .collect::<Vec<_>>();
    ValueRange::collapse_overlapping_ranges(&mut ranges);
    input
        .nearby_tickets
        .iter()
        .map(|t| {
            t.values
                .iter()
                .copied()
                .filter(|&val| !ranges.iter().any(|r| r.contains(val)))
                .map(u32::from)
                .sum::<u32>()
        })
        .sum()
}

#[aoc(day16, part2)]
fn part_2(input: &Input) -> u64 {
    let mut ranges = input
        .fields
        .iter()
        .flat_map(|f| f.valid_ranges)
        .collect::<Vec<_>>();
    ValueRange::collapse_overlapping_ranges(&mut ranges);
    let mapping = determine_column_field_mapping(input, &ranges);
    input
        .your_ticket
        .values
        .iter()
        .enumerate()
        .filter_map(|(column_ix, &val)| {
            let field_ix = mapping[column_ix];
            (input.fields[field_ix].category == Category::Departure).then_some(u64::from(val))
        })
        .product()
}

fn determine_column_field_mapping(input: &Input, ranges: &[ValueRange]) -> Vec<usize> {
    let n = input.fields.len();
    let full_mask: usize = !(!0 << n);
    let mut compatible_fields = vec![full_mask; n];
    for ticket in &input.nearby_tickets {
        if !ticket.is_valid(ranges) {
            continue;
        }
        for (column, &value) in ticket.values.iter().enumerate() {
            for (field_id, field) in input.fields.iter().enumerate() {
                if !field.valid_ranges.iter().any(|r| r.contains(value)) {
                    compatible_fields[column] &= !(1 << field_id);
                }
            }
        }
    }
    let mut locked = 0;
    while let Some((column, field)) =
        compatible_fields
            .iter()
            .enumerate()
            .find_map(|(ix, &field_mask)| {
                let mask = field_mask & !locked;
                mask.is_power_of_two()
                    .then_some((ix, mask.trailing_zeros()))
            })
    {
        compatible_fields[column] = 1 << field;
        locked |= 1 << field;
    }
    for value in &mut compatible_fields {
        *value = value.trailing_zeros().try_into().unwrap();
    }
    compatible_fields
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
        class: 1-3 or 5-7\n\
        row: 6-11 or 33-44\n\
        seat: 13-40 or 45-50\n\
        \n\
        your ticket:\n\
        7,1,14\n\
        \n\
        nearby tickets:\n\
        7,3,47\n\
        40,4,50\n\
        55,2,20\n\
        38,6,12\
    ";

    const EXAMPLE2: &str = "\
        class: 0-1 or 4-19\n\
        row: 0-5 or 8-19\n\
        seat: 0-13 or 16-19\n\
        \n\
        your ticket:\n\
        11,12,13\n\
        \n\
        nearby tickets:\n\
        3,9,18\n\
        15,1,5\n\
        5,14,9\
    ";

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result.fields,
            [
                Field {
                    category: Category::Other,
                    valid_ranges: [
                        ValueRange { low: 1, high: 3 },
                        ValueRange { low: 5, high: 7 }
                    ]
                },
                Field {
                    category: Category::Other,
                    valid_ranges: [
                        ValueRange { low: 6, high: 11 },
                        ValueRange { low: 33, high: 44 },
                    ]
                },
                Field {
                    category: Category::Other,
                    valid_ranges: [
                        ValueRange { low: 13, high: 40 },
                        ValueRange { low: 45, high: 50 },
                    ]
                },
            ]
        );
        assert_eq!(
            result.your_ticket,
            Ticket {
                values: vec![7, 1, 14]
            }
        );
        assert_eq!(
            result.nearby_tickets,
            [
                Ticket {
                    values: vec![7, 3, 47]
                },
                Ticket {
                    values: vec![40, 4, 50]
                },
                Ticket {
                    values: vec![55, 2, 20]
                },
                Ticket {
                    values: vec![38, 6, 12]
                },
            ]
        );
    }

    #[test]
    fn test_collapse_overlapping_ranges() {
        let input = parse(EXAMPLE1).unwrap();
        let mut ranges = input
            .fields
            .iter()
            .flat_map(|f| f.valid_ranges)
            .collect::<Vec<_>>();
        ValueRange::collapse_overlapping_ranges(&mut ranges);
        assert_eq!(
            ranges,
            [
                ValueRange { low: 1, high: 3 },
                ValueRange { low: 5, high: 11 },
                ValueRange { low: 13, high: 50 },
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let input = parse(EXAMPLE1).unwrap();
        let result = part_1(&input);
        assert_eq!(result, 71);
    }

    #[test]
    fn test_determine_column_field_mapping() {
        let input = parse(EXAMPLE2).unwrap();
        let mut ranges = input
            .fields
            .iter()
            .flat_map(|f| f.valid_ranges)
            .collect::<Vec<_>>();
        ValueRange::collapse_overlapping_ranges(&mut ranges);
        let mapping = determine_column_field_mapping(&input, &ranges);
        assert_eq!(mapping, [1, 0, 2]);
    }

    #[test]
    fn test_part_2() {
        let input = parse(EXAMPLE2).unwrap();
        let result = part_2(&input);
        assert_eq!(result, 1);
    }
}
