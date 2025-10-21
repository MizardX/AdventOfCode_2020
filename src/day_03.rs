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
    Open,
    Tree,
}

impl TryFrom<u8> for Tile {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'#' => Self::Tree,
            b'.' => Self::Open,
            _ => return Err(ParseError::InvalidTile(value as char)),
        })
    }
}

fn count_trees_in_slope(grid: &Grid<Tile>, dx: usize, dy: usize) -> usize {
    (0..grid.height)
        .step_by(dy)
        .filter(|&y| grid[(y, (y / dy * dx) % grid.width)] == Tile::Tree)
        .count()
}

#[aoc_generator(day3)]
fn parse(input: &str) -> Result<Grid<Tile>, ParseError> {
    input.parse()
}

#[aoc(day3, part1)]
fn part_1(grid: &Grid<Tile>) -> usize {
    count_trees_in_slope(grid, 3, 1)
}

#[aoc(day3, part2)]
fn part_2(grid: &Grid<Tile>) -> usize {
    count_trees_in_slope(grid, 1, 1)
        * count_trees_in_slope(grid, 3, 1)
        * count_trees_in_slope(grid, 5, 1)
        * count_trees_in_slope(grid, 7, 1)
        * count_trees_in_slope(grid, 1, 2)
}
