use std::collections::{HashMap, HashSet, VecDeque};
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error("Unknown texture: {0:?}")]
    UnknownTexture(String),
    #[error("Unknown color: {0:?}")]
    UnknownColor(String),
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Texture {
    Bright,
    Clear,
    Dark,
    Dim,
    Dotted,
    Drab,
    Dull,
    Faded,
    Light,
    Mirrored,
    Muted,
    Pale,
    Plaid,
    Posh,
    Shiny,
    Striped,
    Vibrant,
    Wavy,
}

impl FromStr for Texture {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bright" => Self::Bright,
            "clear" => Self::Clear,
            "dark" => Self::Dark,
            "dim" => Self::Dim,
            "dotted" => Self::Dotted,
            "drab" => Self::Drab,
            "dull" => Self::Dull,
            "faded" => Self::Faded,
            "light" => Self::Light,
            "mirrored" => Self::Mirrored,
            "muted" => Self::Muted,
            "pale" => Self::Pale,
            "plaid" => Self::Plaid,
            "posh" => Self::Posh,
            "shiny" => Self::Shiny,
            "striped" => Self::Striped,
            "vibrant" => Self::Vibrant,
            "wavy" => Self::Wavy,
            _ => return Err(ParseError::UnknownTexture(s.to_string())),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Color {
    Aqua,
    Beige,
    Black,
    Blue,
    Bronze,
    Brown,
    Chartreuse,
    Coral,
    Crimson,
    Cyan,
    Fuchsia,
    Gold,
    Gray,
    Green,
    Indigo,
    Lavender,
    Lime,
    Magenta,
    Maroon,
    Olive,
    Orange,
    Plum,
    Purple,
    Red,
    Salmon,
    Silver,
    Tan,
    Teal,
    Tomato,
    Turquoise,
    Violet,
    White,
    Yellow,
}

impl FromStr for Color {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "aqua" => Self::Aqua,
            "beige" => Self::Beige,
            "black" => Self::Black,
            "blue" => Self::Blue,
            "bronze" => Self::Bronze,
            "brown" => Self::Brown,
            "chartreuse" => Self::Chartreuse,
            "coral" => Self::Coral,
            "crimson" => Self::Crimson,
            "cyan" => Self::Cyan,
            "fuchsia" => Self::Fuchsia,
            "gold" => Self::Gold,
            "gray" => Self::Gray,
            "green" => Self::Green,
            "indigo" => Self::Indigo,
            "lavender" => Self::Lavender,
            "lime" => Self::Lime,
            "magenta" => Self::Magenta,
            "maroon" => Self::Maroon,
            "olive" => Self::Olive,
            "orange" => Self::Orange,
            "plum" => Self::Plum,
            "purple" => Self::Purple,
            "red" => Self::Red,
            "salmon" => Self::Salmon,
            "silver" => Self::Silver,
            "tan" => Self::Tan,
            "teal" => Self::Teal,
            "tomato" => Self::Tomato,
            "turquoise" => Self::Turquoise,
            "violet" => Self::Violet,
            "white" => Self::White,
            "yellow" => Self::Yellow,
            _ => return Err(ParseError::UnknownColor(s.to_string())),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Bag(Texture, Color);

impl FromStr for Bag {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (texture, color) = s.split_once(' ').ok_or(ParseError::SyntaxError)?;
        Ok(Self(texture.parse()?, color.parse()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    parent: Bag,
    children: Vec<(usize, Bag)>,
}

impl FromStr for Rule {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (parent, rest) = s
            .split_once(" bags contain ")
            .ok_or(ParseError::SyntaxError)?;
        let parent = parent.parse()?;
        let children_str = rest.strip_suffix('.').ok_or(ParseError::SyntaxError)?;
        let mut children = Vec::new();
        if children_str != "no other bags" {
            for part in children_str.split(", ") {
                let (count, rest) = part.split_once(' ').ok_or(ParseError::SyntaxError)?;
                let child = rest
                    .strip_suffix(" bags")
                    .or_else(|| rest.strip_suffix(" bag"))
                    .ok_or(ParseError::SyntaxError)?;
                children.push((count.parse()?, child.parse()?));
            }
        }
        Ok(Self { parent, children })
    }
}

#[aoc_generator(day7)]
fn parse(input: &str) -> Result<Vec<Rule>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day7, part1)]
fn part_1(rules: &[Rule]) -> usize {
    let mut reverse = HashMap::<Bag, Vec<Bag>>::new();
    for rule in rules {
        for &(_, child) in &rule.children {
            reverse.entry(child).or_default().push(rule.parent);
        }
    }
    for parents in reverse.values_mut() {
        parents.sort_unstable();
        parents.dedup();
    }
    let mut seen = HashSet::new();
    let mut pending = VecDeque::new();
    pending.push_back(Bag(Texture::Shiny, Color::Gold));
    while let Some(bag) = pending.pop_front() {
        if !seen.insert(bag) {
            continue;
        }
        for &parent in reverse.get(&bag).iter().copied().flatten() {
            pending.push_back(parent);
        }
    }
    seen.len() - 1 // Except the shiny gold itself
}

#[aoc(day7, part2)]
fn part_2(rules: &[Rule]) -> usize {
    let lookup = rules
        .iter()
        .map(|r| (r.parent, &r.children))
        .collect::<HashMap<_, _>>();
    let mut pending = VecDeque::new();
    pending.push_back((1, Bag(Texture::Shiny, Color::Gold)));
    let mut total = 0;
    while let Some((count, bag)) = pending.pop_front() {
        total += count;
        for &(mult, child) in lookup[&bag] {
            pending.push_back((count * mult, child));
        }
    }
    total - 1 // Except the shiny gold itself
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        light red bags contain 1 bright white bag, 2 muted yellow bags.\n\
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.\n\
        bright white bags contain 1 shiny gold bag.\n\
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.\n\
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.\n\
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.\n\
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.\n\
        faded blue bags contain no other bags.\n\
        dotted black bags contain no other bags.\
    ";

    const EXAMPLE2: &str = "\
        shiny gold bags contain 2 dark red bags.\n\
        dark red bags contain 2 dark orange bags.\n\
        dark orange bags contain 2 dark yellow bags.\n\
        dark yellow bags contain 2 dark green bags.\n\
        dark green bags contain 2 dark blue bags.\n\
        dark blue bags contain 2 dark violet bags.\n\
        dark violet bags contain no other bags.\
    ";

    macro_rules! bag {
        ($texture:ident $color:ident) => {
            Bag(Texture::$texture, Color::$color)
        };
    }

    macro_rules! rule {
        ($texture:ident $color:ident => $($num:literal $child_texture:ident $child_color:ident),* $(,)?) => {
            Rule { parent: bag!($texture $color), children: vec![$(($num, bag!($child_texture $child_color))),*]}
        };
        ($texture:ident $color:ident) => {
            Rule { parent: bag!($texture $color), children: vec![]}
        };
    }

    #[test]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result,
            [
                rule!(Light Red => 1 Bright White, 2 Muted Yellow),
                rule!(Dark Orange => 3 Bright White, 4 Muted Yellow),
                rule!(Bright White => 1 Shiny Gold),
                rule!(Muted Yellow => 2 Shiny Gold, 9 Faded Blue),
                rule!(Shiny Gold => 1 Dark Olive, 2 Vibrant Plum),
                rule!(Dark Olive => 3 Faded Blue, 4 Dotted Black),
                rule!(Vibrant Plum => 5 Faded Blue, 6 Dotted Black),
                rule!(Faded Blue),
                rule!(Dotted Black),
            ]
        );
    }

    #[test_case(EXAMPLE1 => 4)]
    fn test_part_1(input: &str) -> usize {
        let rules = parse(input).unwrap();
        part_1(&rules)
    }

    #[test_case(EXAMPLE1 => 32)]
    #[test_case(EXAMPLE2 => 126)]
    fn test_part_2(input: &str) -> usize {
        let rules = parse(input).unwrap();
        part_2(&rules)
    }
}
