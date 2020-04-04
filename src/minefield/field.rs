use super::grid;
use rand;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Cell {
    /// Represents a bomb
    Bomb,
    /// Clean cell, with X neighbours (0<=X<=8)
    Clean(u8),
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Cell::Bomb => write!(f, "X"),
            Cell::Clean(i) => write!(f, "{}", i),
        }
    }
}

/// Stateless minefield
#[derive(Debug)]
pub struct Minefield {
    field: grid::Grid<Cell>,
    num_bombs: usize,
}

impl Minefield {
    pub fn new(height: usize, width: usize, num_bombs: usize) -> Result<Minefield, String> {
        //! Creates a minefield with the required size and number of bombs
        let empty_field = vec![Cell::Clean(0); height * width];
        let mut field_grid = grid::Grid::new(height, width, empty_field)?;
        let mut rng = rand::thread_rng();
        let bomb_indices_1d = rand::seq::index::sample(&mut rng, field_grid.len(), num_bombs);
        // let bomb_indices_1d = (0..num_bombs * 3).step_by(3);
        // for bomb_ix in bomb_indices.iter() {
        let bomb_indices_2d: Vec<(usize, usize)> = bomb_indices_1d
            .iter()
            .map(|i| field_grid.index_rev(i).unwrap())
            .collect();
        Minefield::bury_bombs(&mut field_grid, &bomb_indices_2d)?;
        Ok(Minefield {
            field: field_grid,
            num_bombs: num_bombs,
        })
    }
    fn bury_bombs(
        field_grid: &mut grid::Grid<Cell>,
        bomb_locations: &[(usize, usize)],
    ) -> Result<(), String> {
        for &(i_bomb, j_bomb) in bomb_locations {
            field_grid.set(i_bomb, j_bomb, Cell::Bomb)?;
            let neighbours = field_grid.neighbours8(i_bomb, j_bomb);
            for (i_n, j_n) in neighbours {
                if let Some(Cell::Clean(i)) = field_grid.get(i_n, j_n) {
                    field_grid.set(i_n, j_n, Cell::Clean(i + 1))?;
                }
            }
        }
        Ok(())
    }

    pub fn dig(&self, row: usize, col: usize) -> Option<Cell> {
        //! Query the status of a cell
        //!
        //! Game is lost if returned cell is a bomb
        //! None is returned if cell is outside of minefield
        self.field.get(row, col)
    }
    pub fn submit(&self, bomb_locations: &[(usize, usize)]) -> bool {
        //! Submit a list of bombs
        //!
        //! Game is won is that list matches **all** the bomb locations
        //! Strategy:
        //! - create a set of unique bomb locations
        //! - check that this set has the required number of locations
        //! - check that every location is effectively a bomb
        let mut bomb_submission = std::collections::HashSet::with_capacity(self.num_bombs);
        for loc in bomb_locations {
            bomb_submission.insert(*loc);
        }
        if bomb_submission.len() != self.num_bombs {
            return false;
        }

        for (row, col) in bomb_submission.iter() {
            if self.dig(*row, *col) != Some(Cell::Bomb) {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Display for Minefield {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.field)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let height = 4;
        let width = 4;
        let expected_num_bombs = 5;
        let field = Minefield::new(height, width, expected_num_bombs).unwrap();
        let mut actual_num_bombs = 0;
        for i in 0..height {
            for j in 0..width {
                match field.dig(i, j) {
                    Some(Cell::Bomb) => actual_num_bombs += 1,
                    Some(Cell::Clean(_)) => (),
                    None => panic!("Digging legal cell was denied: {}, {}", i, j),
                }
            }
        }
        assert_eq!(actual_num_bombs, expected_num_bombs);
    }

    #[test]
    fn preset_example() {
        let grid = grid::Grid::new(
            2,
            2,
            vec![Cell::Bomb, Cell::Bomb, Cell::Clean(2), Cell::Clean(2)],
        )
        .unwrap();

        let field = Minefield {
            field: grid,
            num_bombs: 2,
        };

        assert_eq!(field.dig(0, 0), Some(Cell::Bomb));
        assert_eq!(field.dig(1, 1), Some(Cell::Clean(2)));

        assert_eq!(field.submit(&vec![(0, 0), (0, 1)]), true);
        assert_eq!(field.submit(&vec![(0, 1), (0, 0)]), true);
        assert_eq!(field.submit(&vec![(0, 0)]), false);
        assert_eq!(field.submit(&vec![(0, 0), (0, 0)]), false);
        assert_eq!(field.submit(&vec![(0, 0), (0, 1), (1, 0)]), false);
    }

    fn cell_pattern(pattern: &str) -> Vec<Cell> {
        let mut out: Vec<Cell> = vec![];
        for c in pattern.chars() {
            let car = match c {
                'X' => Some(Cell::Bomb),
                '0'..='8' => Some(Cell::Clean(c.to_digit(10).unwrap() as u8)),
                _ => None,
            };
            if let Some(cell) = car {
                out.push(cell);
            }
        }
        out
    }

    #[test]
    fn bury_bombs_1() {
        let mut grid = grid::Grid::new(3, 3, vec![Cell::Clean(0); 9]).unwrap();
        let bomb_locations: Vec<(usize, usize)> = vec![(1, 1)];
        Minefield::bury_bombs(&mut grid, &bomb_locations).unwrap();
        let expected_grid = cell_pattern("1111X1111");
        assert_eq!(grid.data(), &expected_grid[..]);
    }

    #[test]
    fn bury_bombs_2() {
        let mut grid = grid::Grid::new(3, 3, vec![Cell::Clean(0); 9]).unwrap();
        let bomb_locations: Vec<(usize, usize)> = vec![(0, 0), (1, 1), (2, 2)];
        Minefield::bury_bombs(&mut grid, &bomb_locations).unwrap();
        let expected_grid = cell_pattern("X212X212X");
        assert_eq!(grid.data(), &expected_grid[..]);
    }

    #[test]
    fn bury_bombs_3() {
        let mut grid = grid::Grid::new(3, 5, vec![Cell::Clean(0); 15]).unwrap();
        let bomb_locations: Vec<(usize, usize)> = vec![(0, 0), (0, 3), (1, 1), (1, 4), (2, 2)];

        // pattern for first 3 bombs
        // X__X_    X22X1
        // _X___ -> 2X211
        // _____    11100
        Minefield::bury_bombs(&mut grid, &bomb_locations[..3]).unwrap();
        let expected_grid = cell_pattern("X22X12X21111100");
        assert_eq!(grid.data(), &expected_grid[..]);

        // pattern
        // X__X_    X22X2
        // _X__X -> 2X33X
        // __X__    12X21
        Minefield::bury_bombs(&mut grid, &bomb_locations[3..]).unwrap();
        let expected_grid = cell_pattern("X22X22X33X12X21");
        assert_eq!(grid.data(), &expected_grid[..]);
    }
}
