use std::fmt::{Display, Write};
use std::num::ParseIntError;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

#[derive(Debug, Clone)]
struct Grid<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Grid<T> {
    fn new(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        let data = (0..width * height).map(|_| T::default()).collect();
        Self {
            data,
            width,
            height,
        }
    }

    fn splice(
        &mut self,
        dest_pos: [usize; 2],
        source: &Self,
        source_pos: [usize; 2],
        size: [usize; 2],
    ) where
        T: Copy,
    {
        if dest_pos[0] + size[0] > self.width
            || dest_pos[1] + size[1] > self.height
            || source_pos[0] + size[0] > source.width
            || source_pos[1] + size[1] > source.height
        {
            panic!(
                "Index out of range. Copy {source_pos:?}..{:?} (of {}x{}) into {dest_pos:?}..{:?} (of {}x{})",
                [source_pos[0] + size[0], source_pos[1] + size[1]],
                source.width,
                source.height,
                [dest_pos[0] + size[0], dest_pos[1] + size[1]],
                self.width,
                self.height
            );
        }
        for y in 0..size[1] {
            let dest_start = dest_pos[0] + self.width * (dest_pos[1] + y);
            let dest_end = dest_pos[0] + size[0] + self.width * (dest_pos[1] + y);
            let source_start = source_pos[0] + source.width * (source_pos[1] + y);
            let source_end = source_pos[0] + size[0] + source.width * (source_pos[1] + y);
            self.data[dest_start..dest_end].copy_from_slice(&source.data[source_start..source_end]);
        }
    }

    fn transform(&mut self, mirror_x: bool, mirror_y: bool, transpose: bool) {
        // mirror_x is "reverse rows"
        // mirror_y is "reverse rows" + "full reverse"
        // mirror_x + mirror_y cancels out the "reverse rows", and becomes just "full reverse"
        if mirror_x ^ mirror_y {
            for row in self.data.chunks_mut(self.width) {
                row.reverse();
            }
        }
        if mirror_y {
            self.data.reverse();
        }
        if transpose {
            if self.width == self.height {
                for y in 0..self.height {
                    for x in y + 1..self.width {
                        self.data.swap(x + y * self.width, y + x * self.width);
                    }
                }
            } else {
                unimplemented!(
                    "Transpose for non-square grids: {} x {}",
                    self.width,
                    self.height
                );
            }
        }
    }
}

impl<T> Index<[usize; 2]> for Grid<T> {
    type Output = T;

    fn index(&self, [x, y]: [usize; 2]) -> &Self::Output {
        let ix = x + self.width * y;
        &self.data[ix]
    }
}

impl<T> IndexMut<[usize; 2]> for Grid<T> {
    fn index_mut(&mut self, [x, y]: [usize; 2]) -> &mut Self::Output {
        let ix = x + self.width * y;
        &mut self.data[ix]
    }
}

impl<T> Display for Grid<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.chunks(self.width) {
            for ch in row {
                ch.fmt(f)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Pixel {
    #[default]
    Off,
    On,
}

impl TryFrom<u8> for Pixel {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            b'.' => Self::Off,
            b'#' => Self::On,
            _ => return Err(ParseError::SyntaxError)?,
        })
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Off => '.',
            Self::On => '#',
        })
    }
}

#[derive(Debug, Clone)]
struct Tile {
    id: u64,
    grid: Grid<Pixel>,
}

impl Tile {
    fn border_masks(&self) -> [u16; 8] {
        let width = self.grid.width;
        let height = self.grid.height;
        let mut masks = [0; 8];
        for x in 0..width {
            if self.grid[[x, 0]] == Pixel::On {
                masks[0] |= 1 << x;
                masks[1] |= 1 << (width - 1 - x);
            }
            if self.grid[[x, height - 1]] == Pixel::On {
                masks[2] |= 1 << x;
                masks[3] |= 1 << (width - 1 - x);
            }
        }
        for y in 0..height {
            if self.grid[[0, y]] == Pixel::On {
                masks[4] |= 1 << y;
                masks[5] |= 1 << (height - 1 - y);
            }
            if self.grid[[width - 1, y]] == Pixel::On {
                masks[6] |= 1 << y;
                masks[7] |= 1 << (height - 1 - y);
            }
        }
        masks
    }
}

impl FromStr for Tile {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let id: u64 = lines
            .next()
            .ok_or(ParseError::SyntaxError)?
            .strip_prefix("Tile ")
            .ok_or(ParseError::SyntaxError)?
            .strip_suffix(':')
            .ok_or(ParseError::SyntaxError)?
            .parse()?;
        let mut scan = lines.clone();
        let width = scan.next().unwrap().len();
        let height = 1 + scan.count();
        let mut grid = Grid::new(width, height);
        for (y, line) in lines.enumerate() {
            for (x, ch) in line.bytes().enumerate() {
                grid[[x, y]] = ch.try_into()?;
            }
        }
        Ok(Self { id, grid })
    }
}

#[aoc_generator(day20)]
fn parse(input: &str) -> Result<Vec<Tile>, ParseError> {
    input.split("\n\n").map(str::parse).collect()
}

#[aoc(day20, part1)]
fn part_1(tiles: &[Tile]) -> u64 {
    let (_, neighbors) = get_frames_and_neighbors(tiles);

    tiles
        .iter()
        .zip(&neighbors)
        .filter_map(|(tile, neighbors)| (neighbors.len() == 2).then_some(tile.id))
        .product()
}

#[aoc(day20, part2)]
fn part_2(tiles: &[Tile]) -> usize {
    let (frames, neighbors) = get_frames_and_neighbors(tiles);

    let size = tiles.len().isqrt();
    let placement = place_tiles(&neighbors, size);
    let orientation = orient_tiles(&placement, &frames);

    let large_grid = construct_combiend_grid(tiles, &placement, &orientation);
    let large_grid_pixels = large_grid
        .data
        .iter()
        .filter(|pix| **pix == Pixel::On)
        .count();

    let (max_monster_count, monster_size) = count_monsters(large_grid);
    large_grid_pixels - max_monster_count * monster_size
}

fn get_frames_and_neighbors(tiles: &[Tile]) -> (Vec<Vec<u16>>, Vec<Vec<usize>>) {
    let mut neighbors = vec![vec![]; tiles.len()];
    let mut borders = vec![vec![]; tiles.len()];
    for (ix1, tile1) in tiles.iter().enumerate() {
        let borders1 = tile1.border_masks();
        borders[ix1].extend_from_slice(&borders1);
        for ix2 in 0..ix1 {
            if borders[ix2]
                .iter()
                .any(|e1| borders1.iter().any(|e2| e2 == e1))
            {
                neighbors[ix1].push(ix2);
                neighbors[ix2].push(ix1);
            }
        }
    }
    (borders, neighbors)
}

fn get_corners_and_edges(neighbors: &[Vec<usize>]) -> (Vec<usize>, Vec<usize>) {
    let mut corners = Vec::new();
    let mut edges = Vec::new();
    for (ix, neighbors) in neighbors.iter().enumerate() {
        match neighbors.len() {
            2 => corners.push(ix),
            3 => edges.push(ix),
            _ => (),
        }
    }
    (corners, edges)
}

fn place_tiles(neighbors: &[Vec<usize>], size: usize) -> Grid<usize> {
    let (corners, edges) = get_corners_and_edges(neighbors);

    let mut placement = Grid::new(size, size);

    placement[[0, 0]] = corners[0]; // any corner

    for x in 1..size - 1 {
        placement[[x, 0]] = edges
            .iter()
            .copied()
            .find(|&ix| {
                (x < 2 || ix != placement[[x - 2, 0]])
                    && neighbors[ix].contains(&placement[[x - 1, 0]])
            })
            .unwrap();
    }

    placement[[size - 1, 0]] = corners
        .iter()
        .copied()
        .find(|&ix| {
            ix != placement[[size - 3, 0]] && neighbors[ix].contains(&placement[[size - 2, 0]])
        })
        .unwrap();

    for y in 1..size {
        placement[[0, y]] = edges
            .iter()
            .chain(&corners)
            .copied()
            .find(|&ix| {
                (y < 2 || ix != placement[[0, y - 2]])
                    && neighbors[ix].contains(&placement[[0, y - 1]])
                    && ix != placement[[1, 0]]
            })
            .unwrap();
        for x in 1..size {
            placement[[x, y]] = neighbors[placement[[x - 1, y]]]
                .iter()
                .copied()
                .find(|&ix| {
                    ix != placement[[x - 1, y - 1]]
                        && neighbors[placement[[x, y - 1]]].contains(&ix)
                })
                .unwrap();
        }
    }
    placement
}

fn orient_tiles(placement: &Grid<usize>, frames: &[Vec<u16>]) -> Grid<usize> {
    let size = placement.width;
    let mut orientation = Grid::new(size, size);
    for y in 0..size {
        for x in 0..size {
            //               0        1           2           3          4         5           6          7
            // frames are: [top, top_reversed, bottom, bottom_reversed, left, left_reversed, right, right_reversed]
            // tr my mx => top right bottom left
            // 0  0  0      0    6     2      4
            // 0  0  1      1    4     3      6
            // 0  1  0      2    7     0      5
            // 0  1  1      3    5     1      7
            // 1  0  0      4    2     6      0
            // 1  0  1      6    3     4      1
            // 1  1  0      5    0     7      2
            // 1  1  1      7    1     5      3
            let cur_frames = frames[placement[[x, y]]].as_slice();
            orientation[[x, y]] = (0..8_usize)
                .find(|&ix| {
                    let top = cur_frames[[0, 1, 2, 3, 4, 6, 5, 7][ix]];
                    let right = cur_frames[[6, 4, 7, 5, 2, 3, 0, 1][ix]];
                    let bottom = cur_frames[[2, 3, 0, 1, 6, 4, 7, 5][ix]];
                    let left = cur_frames[[4, 6, 5, 7, 0, 1, 2, 3][ix]];
                    (y == 0 || frames[placement[[x, y - 1]]].contains(&top))
                        && (x == 0 || frames[placement[[x - 1, y]]].contains(&left))
                        && (y == size - 1 || frames[placement[[x, y + 1]]].contains(&bottom))
                        && (x == size - 1 || frames[placement[[x + 1, y]]].contains(&right))
                })
                .unwrap();
        }
    }
    orientation
}

fn construct_combiend_grid(
    tiles: &[Tile],
    placement: &Grid<usize>,
    orientation: &Grid<usize>,
) -> Grid<Pixel> {
    let size = placement.width;

    let tile_width = tiles[0].grid.width;
    let tile_height = tiles[0].grid.height;
    let large_grid_size = (tile_width - 2) * size;
    let mut large_grid = Grid::new(large_grid_size, large_grid_size);
    for y in 0..size {
        for x in 0..size {
            // TODO: Could we avoid allocating a clone? Maybe update splice() to account for transformations?
            let mut oriented = tiles[placement[[x, y]]].grid.clone();
            let ord = orientation[[x, y]];
            oriented.transform((ord & 1) != 0, (ord & 2) != 0, (ord & 4) != 0);
            large_grid.splice(
                [x * (tile_width - 2), y * (tile_height - 2)],
                &oriented,
                [1, 1],
                [tile_width - 2, tile_height - 2],
            );
        }
    }
    large_grid
}

fn count_monsters(mut large_grid: Grid<Pixel>) -> (usize, usize) {
    let (monster_width, monster_height, monster_pixels) = get_monster();
    let large_grid_size = large_grid.width;

    let mut monster_count = 0;
    for [transpose, mirror_x, mirror_y] in [
        // Gray code to try every orientation
        [false, false, false],
        [false, false, true],
        [false, true, false],
        [false, false, true],
        [true, false, false],
        [false, false, true],
        [false, true, false],
        [false, false, true],
    ] {
        large_grid.transform(mirror_x, mirror_y, transpose);
        for y in 0..=(large_grid_size - monster_height) {
            'next_position: for x in 0..=(large_grid_size - monster_width) {
                for &(dx, dy) in &monster_pixels {
                    if large_grid[[x + dx, y + dy]] != Pixel::On {
                        continue 'next_position;
                    }
                }
                monster_count += 1;
            }
        }
        if monster_count > 0 {
            break;
        }
    }
    (monster_count, monster_pixels.len())
}

fn get_monster() -> (usize, usize, Vec<(usize, usize)>) {
    let monster_image = "\
        ..................#.\n\
        #....##....##....###\n\
        .#..#..#..#..#..#...\
    ";
    let monster_width = monster_image.lines().map(str::len).max().unwrap();
    let monster_height = monster_image.lines().count();
    let monster_pixels = monster_image
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.bytes()
                .enumerate()
                .filter_map(move |(x, ch)| (ch == b'#').then_some((x, y)))
        })
        .collect::<Vec<_>>();
    (monster_width, monster_height, monster_pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
        Tile 2311:\n\
        ..##.#..#.\n\
        ##..#.....\n\
        #...##..#.\n\
        ####.#...#\n\
        ##.##.###.\n\
        ##...#.###\n\
        .#.#.#..##\n\
        ..#....#..\n\
        ###...#.#.\n\
        ..###..###\n\
        \n\
        Tile 1951:\n\
        #.##...##.\n\
        #.####...#\n\
        .....#..##\n\
        #...######\n\
        .##.#....#\n\
        .###.#####\n\
        ###.##.##.\n\
        .###....#.\n\
        ..#.#..#.#\n\
        #...##.#..\n\
        \n\
        Tile 1171:\n\
        ####...##.\n\
        #..##.#..#\n\
        ##.#..#.#.\n\
        .###.####.\n\
        ..###.####\n\
        .##....##.\n\
        .#...####.\n\
        #.##.####.\n\
        ####..#...\n\
        .....##...\n\
        \n\
        Tile 1427:\n\
        ###.##.#..\n\
        .#..#.##..\n\
        .#.##.#..#\n\
        #.#.#.##.#\n\
        ....#...##\n\
        ...##..##.\n\
        ...#.#####\n\
        .#.####.#.\n\
        ..#..###.#\n\
        ..##.#..#.\n\
        \n\
        Tile 1489:\n\
        ##.#.#....\n\
        ..##...#..\n\
        .##..##...\n\
        ..#...#...\n\
        #####...#.\n\
        #..#.#.#.#\n\
        ...#.#.#..\n\
        ##.#...##.\n\
        ..##.##.##\n\
        ###.##.#..\n\
        \n\
        Tile 2473:\n\
        #....####.\n\
        #..#.##...\n\
        #.##..#...\n\
        ######.#.#\n\
        .#...#.#.#\n\
        .#########\n\
        .###.#..#.\n\
        ########.#\n\
        ##...##.#.\n\
        ..###.#.#.\n\
        \n\
        Tile 2971:\n\
        ..#.#....#\n\
        #...###...\n\
        #.#.###...\n\
        ##.##..#..\n\
        .#####..##\n\
        .#..####.#\n\
        #..#.#..#.\n\
        ..####.###\n\
        ..#.#.###.\n\
        ...#.#.#.#\n\
        \n\
        Tile 2729:\n\
        ...#.#.#.#\n\
        ####.#....\n\
        ..#.#.....\n\
        ....#..#.#\n\
        .##..##.#.\n\
        .#.####...\n\
        ####.#.#..\n\
        ##.####...\n\
        ##..#.##..\n\
        #.##...##.\n\
        \n\
        Tile 3079:\n\
        #.#.#####.\n\
        .#..######\n\
        ..#.......\n\
        ######....\n\
        ####.#..#.\n\
        .#...#.##.\n\
        #.#####.##\n\
        ..#.###...\n\
        ..#.......\n\
        ..#.###...\
    ";

    #[test]
    fn test_part_1() {
        let tiles = parse(EXAMPLE).unwrap();
        let result = part_1(&tiles);
        assert_eq!(result, 20_899_048_083_289);
    }

    #[test]
    fn test_part_2() {
        let tiles = parse(EXAMPLE).unwrap();
        let result = part_2(&tiles);
        assert_eq!(result, 273);
    }
}
