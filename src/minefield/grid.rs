use std::fmt;
use std::iter::IntoIterator;
use std::usize::{self};

pub struct Grid<T> {
    height: usize,
    width: usize,
    data: Vec<T>,
}

pub struct IterGrid<'a, T: 'a> {
    grid: &'a Grid<T>,
    curr_ix: usize,
}

impl<'a, T> Iterator for IterGrid<'a, T> {
    type Item = (usize, usize, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.grid.index_rev(self.curr_ix) {
            Some((i, j)) => {
                let value = &self.grid.data[self.curr_ix];
                self.curr_ix += 1;
                Some((i, j, value))
            }
            None => None,
        }
    }
}

impl<'a, T> Grid<T> {
    pub fn len(&self) -> usize {
        self.height * self.width
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    pub fn index(&self, row: usize, col: usize) -> Option<usize> {
        if (row <= self.height - 1) && (col <= self.width - 1) {
            Some(row * self.width + col)
        } else {
            None
        }
    }

    pub fn index_rev(&self, index: usize) -> Option<(usize, usize)> {
        if index < self.len() {
            Some((index / self.width, index % self.width))
        } else {
            None
        }
    }

    pub fn iter(&'a self) -> IterGrid<'a, T> {
        IterGrid {
            grid: &self,
            curr_ix: 0,
        }
    }

    pub fn neighbours8(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let mut neighbours: Vec<(usize, usize)> = vec![];
        let row_left: usize = if row > 0 { row - 1 } else { row };
        let row_right: usize = if row < self.height - 1 { row + 1 } else { row };
        let col_left: usize = if col > 0 { col - 1 } else { col };
        let col_right: usize = if col < self.width - 1 { col + 1 } else { col };
        for i in row_left..=row_right {
            for j in col_left..=col_right {
                if i != row || j != col {
                    neighbours.push((i, j))
                }
            }
        }
        neighbours
    }
}

impl<T: Copy> Grid<T> {
    pub fn new(height: usize, width: usize, data: Vec<T>) -> Result<Grid<T>, String> {
        let size = height * width;
        if size == data.len() {
            Ok(Grid {
                height: height,
                width: width,
                data: data,
            })
        } else {
            Err(String::from(
                "Data length is incompatible with given height and width",
            ))
        }
    }

    pub fn data(&self) -> &[T] {
        return &self.data;
    }

    pub fn get(&self, row: usize, col: usize) -> Option<T> {
        let index = self.index(row, col)?;
        Some(self.data[index])
    }

    pub fn set(&mut self, row: usize, col: usize, value: T) -> Result<(), String> {
        let index = self.index(row, col);
        match index {
            Some(i) => {
                *self.data.get_mut(i).unwrap() = value;
                Ok(())
            }
            None => Err(format!("Non-existent index requested: ({}, {})", row, col)),
        }
    }
}

impl<T> fmt::Debug for Grid<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "grid of size h={}, w={}\n{:?}",
            self.height, self.width, self.data
        )
    }
}

impl<T> fmt::Display for Grid<T>
where
    T: fmt::Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for row in 0..self.height {
            for col in 0..self.width {
                let sep = if col < self.width - 1 { " " } else { "" };
                write!(f, "{}{}", self.get(row, col).unwrap(), sep)?
            }
            if row < self.height - 1 {
                write!(f, "\n")?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn index() {
        let (height, width) = (3, 5);
        let data = vec![0; height * width];

        let grid = Grid::new(height, width, data).unwrap();
        assert_eq!(grid.shape(), (height, width));
        assert_eq!(grid.len(), height * width);

        for i in 0..height {
            for j in 0..width {
                let tt_index = grid.index(i, j).unwrap();
                let (i_rev, j_rev) = grid.index_rev(tt_index).unwrap();
                assert_eq!((i, j), (i_rev, j_rev), "Wrong index_rev for t={}", tt_index);
            }
        }
    }

    #[test]
    fn get_set() {
        let data = vec![1, 2, 3, 4];

        let mut grid = Grid::new(2, 2, data).unwrap();
        assert_eq!(grid.shape(), (2, 2));

        assert_eq!(grid.get(0, 0), Some(1));
        assert_eq!(grid.get(0, 1), Some(2));
        assert_eq!(grid.get(1, 0), Some(3));
        assert_eq!(grid.get(3, 0), None);

        assert_eq!(grid.set(0, 0, 10), Ok(()));
        assert_eq!(grid.get(0, 0), Some(10));
    }

    #[test]
    fn neighbours() {
        let data = vec![0; 12];
        let grid = Grid::new(4, 3, data).unwrap();
        assert_eq!(
            grid.neighbours8(1, 1),
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (1, 0),
                (1, 2),
                (2, 0),
                (2, 1),
                (2, 2)
            ]
        );
        assert_eq!(grid.neighbours8(0, 0), vec![(0, 1), (1, 0), (1, 1)]);
        assert_eq!(grid.neighbours8(3, 2), vec![(2, 1), (2, 2), (3, 1)]);
    }

    #[test]
    fn display() {
        let data = vec![1, 2, 3, 4];
        let grid = Grid::new(2, 2, data).unwrap();
        assert_eq!(format!("{}", grid), "1 2\n3 4");
    }

    #[test]
    fn iter() {
        let (h, w) = (4, 3);
        let data = (0..h * w).collect::<Vec<usize>>();
        let grid = Grid::new(h, w, data).unwrap();
        let mut grid_iter = grid.iter();
        for e_i in 0..h {
            for e_j in 0..w {
                let expected_value = grid.index(e_i, e_j).unwrap();
                let (i, j, &val) = grid_iter.next().unwrap();
                assert_eq!((e_i, e_j), (i, j), "Grid indices don't match");
                assert_eq!(expected_value, val, "Grid values don't match");
            }
        }
        assert_eq!(grid_iter.next(), None);
    }
}
