use std::fmt::{Display, Write};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid tile: {0:?}")]
    InvalidTile(char),
}

#[derive(Debug, Clone)]
struct Grid<T> {
    data: Vec<T>,
    stride: usize,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    fn new_default(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        let data = (0..width * height).map(|_| T::default()).collect();
        Self {
            data,
            stride: width,
            width,
            height,
        }
    }
}

impl<T> FromStr for Grid<T>
where
    T: TryFrom<u8> + Default,
    ParseError: From<T::Error>,
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let width = s.lines().next().unwrap().len();
        let height = s.lines().count();
        let mut grid = Self {
            data: (0..width * height).map(|_| T::default()).collect(),
            stride: width,
            width,
            height,
        };
        for (r, line) in s.lines().enumerate() {
            for (c, ch) in line.bytes().enumerate() {
                grid[(r, c)] = ch.try_into()?;
            }
        }
        Ok(grid)
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for row in self.data.chunks(self.stride) {
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }
            for cell in &row[..self.width] {
                write!(f, "{cell}")?;
            }
        }
        Ok(())
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        assert!(
            index.0 < self.height && index.1 < self.width,
            "Index out of range: {index:?} < ({}, {})",
            self.height,
            self.width
        );
        &self.data[index.0 * self.stride + index.1]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        assert!(
            index.0 < self.height && index.1 < self.width,
            "Index out of range: {index:?} < ({}, {})",
            self.height,
            self.width
        );
        &mut self.data[index.0 * self.stride + index.1]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    #[default]
    Floor,
    Empty,
    Occupied,
}

impl TryFrom<u8> for Tile {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'.' => Self::Floor,
            b'#' => Self::Occupied,
            b'L' => Self::Empty,
            _ => return Err(ParseError::InvalidTile(value as char)),
        })
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Floor => ' ',
            Self::Empty => 'L',
            Self::Occupied => '#',
        })
    }
}

#[derive(Debug, Clone)]
struct Simulation {
    state: Grid<Tile>,
    counts: Grid<u8>,
}

impl Simulation {
    fn new(grid: &Grid<Tile>) -> Self {
        let state = grid.clone();
        let counts = Grid::new_default(state.width, state.height);
        Self { state, counts }
    }

    fn tick(&mut self, strategy: Strategy) -> bool {
        let max_distance = match strategy {
            Strategy::NearIntolerant => 1,
            Strategy::FarTolerant => isize::MAX,
        };
        let &Grid {
            stride,
            width,
            height,
            ..
        } = &self.counts;
        for (y, counts_row) in self.counts.data.chunks_mut(stride).enumerate() {
            for (x, counts) in counts_row[..width].iter_mut().enumerate() {
                *counts = 0;
                for (dx, dy) in [
                    (-1, -1),
                    (-1, 0),
                    (-1, 1),
                    (0, 1),
                    (1, 1),
                    (1, 0),
                    (1, -1),
                    (0, -1),
                ] {
                    for distance in 1..=max_distance {
                        if let Some(x1) = x.checked_add_signed(distance * dx)
                            && x1 < width
                            && let Some(y1) = y.checked_add_signed(distance * dy)
                            && y1 < height
                        {
                            match self.state[(y1, x1)] {
                                Tile::Floor => (),
                                Tile::Empty => break,
                                Tile::Occupied => {
                                    *counts += 1;
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        let mut any_change = false;
        for (counts_row, tiles_row) in self
            .counts
            .data
            .chunks(self.counts.stride)
            .zip(self.state.data.chunks_mut(self.state.stride))
        {
            for (&counts, tile) in counts_row[..width].iter().zip(&mut tiles_row[..width]) {
                let new_tile = match (*tile, counts, strategy) {
                    (Tile::Empty, 0, _) => Tile::Occupied,
                    (Tile::Occupied, 4, Strategy::NearIntolerant) | (Tile::Occupied, 5..=8, _) => {
                        Tile::Empty
                    }
                    (old, _, _) => old,
                };
                any_change = any_change || new_tile != *tile;
                *tile = new_tile;
            }
        }
        any_change
    }
}

#[aoc_generator(day11)]
fn parse(input: &str) -> Result<Grid<Tile>, ParseError> {
    input.parse()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    NearIntolerant,
    FarTolerant,
}

#[aoc(day11, part1)]
fn part_1(seat_layout: &Grid<Tile>) -> usize {
    let mut sim = Simulation::new(seat_layout);
    while sim.tick(Strategy::NearIntolerant) {}
    sim.state
        .data
        .iter()
        .filter(|tile| matches!(tile, Tile::Occupied))
        .count()
}

#[aoc(day11, part2)]
fn part_2(seat_layout: &Grid<Tile>) -> usize {
    let mut sim = Simulation::new(seat_layout);
    while sim.tick(Strategy::FarTolerant) {}
    sim.state
        .data
        .iter()
        .filter(|tile| matches!(tile, Tile::Occupied))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        L.LL.LL.LL\n\
        LLLLLLL.LL\n\
        L.L.L..L..\n\
        LLLL.LL.LL\n\
        L.LL.LL.LL\n\
        L.LLLLL.LL\n\
        ..L.L.....\n\
        LLLLLLLLLL\n\
        L.LLLLLL.L\n\
        L.LLLLL.LL\
    ";

    #[test]
    fn test_part_1() {
        let seat_layout = parse(EXAMPLE).unwrap();
        let result = part_1(&seat_layout);
        assert_eq!(result, 37);
    }

    #[test]
    fn test_part_2() {
        let seat_layout = parse(EXAMPLE).unwrap();
        let result = part_2(&seat_layout);
        assert_eq!(result, 26);
    }
}
