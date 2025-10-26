use std::fmt::{Display, Write};
use std::ops::{Index, IndexMut};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid<T, SIZE> {
    data: Vec<T>,
    sizes: SIZE,
    strides: SIZE,
}

impl<T: Default, const D: usize> Grid<T, [usize; D]> {
    fn new(sizes: [usize; D]) -> Self {
        let data = (0..sizes.iter().copied().product())
            .map(|_| T::default())
            .collect();
        let mut prev = 1;
        let strides = sizes.map(|d| {
            let res = prev;
            prev *= d;
            res
        });
        Self {
            data,
            sizes,
            strides,
        }
    }

    fn reshape<const D1: usize>(
        &self,
        new_size: [usize; D1],
        offset: [usize; D1],
    ) -> Result<Grid<T, [usize; D1]>, RehsapeError>
    where
        T: Clone,
    {
        if D1 < D {
            return Err(RehsapeError::DimensionError);
        }
        let mut new_grid = Grid::new(new_size);
        for (ix, value) in self.data.iter().enumerate() {
            if let Some(new_ix) = self
                .sizes
                .iter()
                .zip(&self.strides)
                .map(|(&sz, &st)| ix / st % sz)
                .chain(std::iter::repeat(0))
                .zip(&new_grid.sizes)
                .zip(&new_grid.strides)
                .zip(&offset)
                .map(|(((ix, &size), &stride), &offset)| {
                    (ix + offset < size).then_some((ix + offset) * stride)
                })
                .sum::<Option<usize>>()
            {
                new_grid.data[new_ix] = value.clone();
            }
        }
        Ok(new_grid)
    }

    fn for_each_neighbor(&self, pos: [usize; D], callback: &mut impl FnMut([usize; D], &T)) {
        fn walk<T, const D: usize>(
            grid: &Grid<T, [usize; D]>,
            mut pos: [usize; D],
            dim: usize,
            callback: &mut impl FnMut([usize; D], &T),
        ) {
            if dim == D {
                callback(pos, &grid[pos]);
                return;
            }
            let low = pos[dim].saturating_sub(1);
            let high = (pos[dim] + 1).min(grid.sizes[dim] - 1);
            for x in low..=high {
                pos[dim] = x;
                walk(grid, pos, dim + 1, callback);
            }
        }
        walk(self, pos, 0, callback);
    }

    fn for_each_cell(&self, callback: &mut impl FnMut([usize; D], &T)) {
        for (ix, val) in self.data.iter().enumerate() {
            let mut pos = [0; D];
            for (dim, (&size, &stride)) in self.sizes.iter().zip(&self.strides).enumerate() {
                pos[dim] = ix / stride % size;
            }
            callback(pos, val);
        }
    }
}

#[derive(Debug, Error)]
enum RehsapeError {
    #[error("Too few dimensions")]
    DimensionError,
}

impl<T, const D: usize> Index<[usize; D]> for Grid<T, [usize; D]> {
    type Output = T;

    fn index(&self, index: [usize; D]) -> &Self::Output {
        let ix = index
            .into_iter()
            .zip(self.strides)
            .zip(self.sizes)
            .map(|((x, stride), size)| (x < size).then_some(x * stride))
            .sum::<Option<usize>>()
            .expect("Index in range");
        &self.data[ix]
    }
}

impl<T, const D: usize> IndexMut<[usize; D]> for Grid<T, [usize; D]> {
    fn index_mut(&mut self, index: [usize; D]) -> &mut Self::Output {
        let ix = index
            .into_iter()
            .zip(self.strides)
            .zip(self.sizes)
            .map(|((x, stride), size)| (x < size).then_some(x * stride))
            .sum::<Option<usize>>()
            .expect("Index in range");
        &mut self.data[ix]
    }
}

impl<T> Display for Grid<T, [usize; 2]>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [width, height] = self.sizes;
        for y in 0..height {
            for x in 0..width {
                self[[x, y]].fmt(f)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<T> Display for Grid<T, [usize; 3]>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [width, height, depth] = self.sizes;
        for y in 0..height {
            for cell_x in 0..depth {
                for x in 0..width {
                    self[[x, y, cell_x]].fmt(f)?;
                }
                f.write_char(' ')?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl<T> Display for Grid<T, [usize; 4]>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [width, height, depth, anakata] = self.sizes;
        for cell_y in 0..anakata {
            for y in 0..height {
                for cell_x in 0..depth {
                    for x in 0..width {
                        self[[x, y, cell_x, cell_y]].fmt(f)?;
                    }
                    f.write_char(' ')?;
                }
                f.write_char('\n')?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Tile {
    #[default]
    Inactive,
    Active,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Inactive => '.',
            Self::Active => '#',
        })
    }
}

#[derive(Debug, Error)]
enum ParseError {
    #[error("Invalid tile")]
    InvalidTile,
}

#[aoc_generator(day17)]
fn parse(input: &str) -> Result<Grid<Tile, [usize; 2]>, ParseError> {
    let height = input.lines().count();
    let width = input.lines().next().unwrap().len();
    let mut grid = Grid::<Tile, [usize; 2]>::new([width, height]);
    for (y, line) in input.lines().enumerate() {
        for (x, ch) in line.bytes().enumerate() {
            grid[[x, y]] = match ch {
                b'#' => Tile::Active,
                b'.' => Tile::Inactive,
                _ => return Err(ParseError::InvalidTile),
            };
        }
    }
    Ok(grid)
}

#[aoc(day17, part1)]
fn part_1(grid: &Grid<Tile, [usize; 2]>) -> usize {
    let [width, height] = grid.sizes;
    let cycles = 6;
    let mut grid = grid
        .reshape(
            [width + 2 * cycles, height + 2 * cycles, 1 + 2 * cycles],
            [cycles, cycles, cycles],
        )
        .unwrap();
    let mut next = grid.clone();
    for _ in 0..cycles {
        grid.for_each_cell(&mut |pos, &center| {
            let mut count_neighbors = 0;
            grid.for_each_neighbor(pos, &mut |npos, &neighbor| {
                count_neighbors += u8::from(npos != pos && neighbor == Tile::Active);
            });
            next[pos] = match (center, count_neighbors) {
                (Tile::Active, 2 | 3) | (Tile::Inactive, 3) => Tile::Active,
                _ => Tile::Inactive,
            };
        });
        (grid, next) = (next, grid);
    }
    let mut count_alive = 0;
    grid.for_each_cell(&mut |_, &value| {
        count_alive += usize::from(value == Tile::Active);
    });
    count_alive
}

#[aoc(day17, part2)]
fn part_2(grid: &Grid<Tile, [usize; 2]>) -> usize {
    let [width, height] = grid.sizes;
    let cycles = 6;
    let mut grid = grid
        .reshape(
            [
                width + 2 * cycles,
                height + 2 * cycles,
                1 + 2 * cycles,
                1 + 2 * cycles,
            ],
            [cycles, cycles, cycles, cycles],
        )
        .unwrap();
    let mut next = grid.clone();
    for _ in 0..cycles {
        grid.for_each_cell(&mut |pos, &center| {
            let mut count_neighbors = 0;
            grid.for_each_neighbor(pos, &mut |npos, &neighbor| {
                count_neighbors += u8::from(npos != pos && neighbor == Tile::Active);
            });
            next[pos] = match (center, count_neighbors) {
                (Tile::Active, 2 | 3) | (Tile::Inactive, 3) => Tile::Active,
                _ => Tile::Inactive,
            };
        });
        (grid, next) = (next, grid);
    }
    let mut count_alive = 0;
    grid.for_each_cell(&mut |_, &value| {
        count_alive += usize::from(value == Tile::Active);
    });
    count_alive
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        .#.\n\
        ..#\n\
        ###\
    ";

    #[test]
    fn test_parse() {
        let grid = parse(EXAMPLE).unwrap();
        assert_eq!(
            grid,
            Grid::<Tile, [usize; 2]> {
                data: vec![
                    Tile::Inactive,
                    Tile::Active,
                    Tile::Inactive,
                    Tile::Inactive,
                    Tile::Inactive,
                    Tile::Active,
                    Tile::Active,
                    Tile::Active,
                    Tile::Active
                ],
                sizes: [3, 3],
                strides: [1, 3],
            }
        );
    }

    #[test]
    fn test_part_1() {
        let grid = parse(EXAMPLE).unwrap();
        let result = part_1(&grid);
        assert_eq!(result, 112);
    }

    #[test]
    fn test_part_2() {
        let grid = parse(EXAMPLE).unwrap();
        let result = part_2(&grid);
        assert_eq!(result, 848);
    }
}
