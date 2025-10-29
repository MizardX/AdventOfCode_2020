use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    East,
    SouthEast,
    SouthWest,
    West,
    NorthWest,
    NorthEast,
}

impl Direction {
    const fn all() -> [Self; 6] {
        [
            Self::East,
            Self::SouthEast,
            Self::SouthWest,
            Self::West,
            Self::NorthWest,
            Self::NorthEast,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
struct HexCoord {
    q: isize,
    r: isize,
}

impl HexCoord {
    const fn s(self) -> isize {
        -self.q - self.r
    }

    fn dist(self) -> usize {
        // equivalent to (abs(q)+abs(r)+abs(s))/2
        self.q
            .unsigned_abs()
            .max(self.r.unsigned_abs())
            .max(self.s().unsigned_abs())
    }

    fn neighbors(self) -> [Self; 6] {
        Direction::all().map(|d| self + d)
    }
}

impl AddAssign<Direction> for HexCoord {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::SouthEast | Direction::SouthWest => self.r += 1,
            Direction::NorthWest | Direction::NorthEast => self.r -= 1,
            Direction::East | Direction::West => {}
        }
        match rhs {
            Direction::East | Direction::NorthEast => self.q += 1,
            Direction::SouthWest | Direction::West => self.q -= 1,
            Direction::SouthEast | Direction::NorthWest => {}
        }
    }
}

impl Add<Direction> for HexCoord {
    type Output = Self;

    fn add(mut self, rhs: Direction) -> Self::Output {
        self += rhs;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    steps: Vec<Direction>,
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxErorr,
}

impl FromStr for Path {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut steps = Vec::new();
        let mut north_south = None;
        for ch in s.bytes() {
            steps.push(match (ch, north_south) {
                (b'n' | b's', None) => {
                    north_south = Some(ch);
                    continue;
                }
                (b'e', None) => Direction::East,
                (b'e', Some(b'n')) => Direction::NorthEast,
                (b'e', Some(b's')) => Direction::SouthEast,
                (b'w', None) => Direction::West,
                (b'w', Some(b'n')) => Direction::NorthWest,
                (b'w', Some(b's')) => Direction::SouthWest,
                _ => return Err(ParseError::SyntaxErorr),
            });
            north_south = None;
        }
        Ok(Self { steps })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Color {
    #[default]
    Dead,
    Alive,
}

impl Color {
    const fn flip(&mut self) {
        *self = match *self {
            Self::Dead => Self::Alive,
            Self::Alive => Self::Dead,
        }
    }
}

#[aoc_generator(day24)]
fn parse(input: &str) -> Result<Vec<Path>, ParseError> {
    input.lines().map(str::parse).collect()
}

#[aoc(day24, part1)]
fn part_1(paths: &[Path]) -> usize {
    let tiles = into_tiles(paths);
    tiles.into_values().filter(|&c| c == Color::Alive).count()
}

fn into_tiles(paths: &[Path]) -> HashMap<HexCoord, Color> {
    let mut tiles = HashMap::<HexCoord, Color>::new();
    for path in paths {
        let mut pos = HexCoord::default();
        for &step in &path.steps {
            pos += step;
        }
        tiles.entry(pos).or_default().flip();
    }
    tiles
}

#[aoc(day24, part2)]
fn part_2(paths: &[Path]) -> usize {
    let mut tiles = into_tiles(paths);
    let mut next = HashMap::new();
    for _day in 1..=100 {
        evolve(&tiles, &mut next);
        (tiles, next) = (next, tiles);
    }
    tiles.len()
}

#[allow(unused)]
fn display(source: &HashMap<HexCoord, Color>) {
    let max_radius = source
        .iter()
        .filter_map(|(pos, &clr)| (clr == Color::Alive).then_some(pos.dist()))
        .max()
        .unwrap()
        .cast_signed();
    for r in (-max_radius..=max_radius).rev() {
        print!("{0:1$}", "", r.unsigned_abs());
        for q in (-max_radius - r).max(-max_radius)..=(max_radius - r).min(max_radius) {
            let pos = HexCoord { q, r };
            match source.get(&pos).copied().unwrap_or_default() {
                Color::Alive => print!("# "),
                Color::Dead => print!(". "),
            }
        }
        println!();
    }
}

fn evolve(source: &HashMap<HexCoord, Color>, dest: &mut HashMap<HexCoord, Color>) {
    dest.clear();
    let max_radius = source
        .iter()
        .filter_map(|(pos, &clr)| (clr == Color::Alive).then_some(pos.dist()))
        .max()
        .unwrap()
        .cast_signed()
        + 1;
    for r in -max_radius..=max_radius {
        for q in -max_radius..=max_radius {
            let pos = HexCoord { q, r };
            if pos.s().abs() > max_radius {
                continue;
            }
            let state = source.get(&pos).copied().unwrap_or_default();
            let alive_neighbors = pos
                .neighbors()
                .into_iter()
                .filter(|n| source.get(n).copied().unwrap_or_default() == Color::Alive)
                .count();
            if matches!(
                (state, alive_neighbors),
                (Color::Dead, 2) | (Color::Alive, 1 | 2)
            ) {
                dest.insert(pos, Color::Alive);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        sesenwnenenewseeswwswswwnenewsewsw\n\
        neeenesenwnwwswnenewnwwsewnenwseswesw\n\
        seswneswswsenwwnwse\n\
        nwnwneseeswswnenewneswwnewseswneseene\n\
        swweswneswnenwsewnwneneseenw\n\
        eesenwseswswnenwswnwnwsewwnwsene\n\
        sewnenenenesenwsewnenwwwse\n\
        wenwwweseeeweswwwnwwe\n\
        wsweesenenewnwwnwsenewsenwwsesesenwne\n\
        neeswseenwwswnwswswnw\n\
        nenwswwsewswnenenewsenwsenwnesesenew\n\
        enewnwewneswsewnwswenweswnenwsenwsw\n\
        sweneswneswneneenwnewenewwneswswnese\n\
        swwesenesewenwneswnwwneseswwne\n\
        enesenwswwswneneswsenwnewswseenwsese\n\
        wnwnesenesenenwwnenwsewesewsesesew\n\
        nenewswnwewswnenesenwnesewesw\n\
        eneswnwswnwsenenwnwnwwseeswneewsenese\n\
        neswnwewnwnwseenwseesewsenwsweewe\n\
        wseweeenwnesenwwwswnew\
    ";

    #[test]
    fn test_parse() {
        const SW: Direction = Direction::SouthWest;
        const W: Direction = Direction::West;
        const NW: Direction = Direction::NorthWest;
        const NE: Direction = Direction::NorthEast;
        const E: Direction = Direction::East;
        const SE: Direction = Direction::SouthEast;
        let result = parse(EXAMPLE).unwrap();
        assert_eq!(result.len(), 20);
        assert_eq!(
            result[0].steps,
            [
                SE, SE, NW, NE, NE, NE, W, SE, E, SW, W, SW, SW, W, NE, NE, W, SE, W, SW
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let paths = parse(EXAMPLE).unwrap();
        let result = part_1(&paths);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_part_2() {
        let paths = parse(EXAMPLE).unwrap();
        let result = part_2(&paths);
        assert_eq!(result, 2208);
    }
}
