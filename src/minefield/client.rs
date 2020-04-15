use super::field::{Cell, Minefield};
use super::grid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Revealed(Cell),
    Hidden,
    Flagged,
    Marked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Running,
    Lost,
    Won,
}

pub struct Client {
    pub minefield: Minefield,
    state: grid::Grid<CellState>,
    game_state: GameState,
}

// split off an arbitrary element from a (non-empty) set
fn pop_from_set<T>(set: &mut std::collections::HashSet<T>) -> T
where
    T: Eq + Clone + std::hash::Hash,
{
    let elt = set.iter().next().cloned().unwrap();
    set.remove(&elt);
    elt
}

impl Client {
    pub fn from_minefield(field: Minefield) -> Client {
        let (height, width) = field.shape();
        let state_init_data = vec![CellState::Hidden; height * width];
        let state = grid::Grid::new(height, width, state_init_data).unwrap();
        Client {
            minefield: field,
            state: state,
            game_state: GameState::Running,
        }
    }

    pub fn new_random(height: usize, width: usize, num_bombs: usize) -> Client {
        let field = Minefield::new(height, width, num_bombs).unwrap();
        Self::from_minefield(field)
    }

    pub fn query_update(&mut self, row: usize, col: usize) -> Cell {
        let cell = self.minefield.dig(row, col).unwrap();
        self.state.set(row, col, CellState::Revealed(cell)).unwrap();
        if cell == Cell::Bomb {
            self.game_state = GameState::Lost;
        }
        cell
    }

    pub fn num_bombs(&self) -> usize {
        return self.minefield.num_bombs();
    }

    pub fn query_smart(&mut self, row: usize, col: usize) -> GameState {
        let mut set = std::collections::HashSet::new();
        set.insert((row, col));
        while !set.is_empty() {
            let (i, j) = pop_from_set(&mut set);
            // print!("Query: {},{}", i, j);
            let c = self.query_update(i, j);
            // println!("-> {:?}", c);
            // println!("Set: {:?}", set);
            if c == Cell::Clean(0) {
                for (i_n, j_n) in self.state.neighbours8(i, j) {
                    if self.state.get(i_n, j_n).unwrap() == CellState::Hidden {
                        set.insert((i_n, j_n));
                    }
                }
            }
        }
        GameState::Running
    }

    pub fn reveal(&mut self, all: bool) {
        let original_game_state = self.game_state;
        let (h, w) = self.state.shape();
        for i in 0..h {
            for j in 0..w {
                let cell_state = self.state.get(i, j).unwrap();
                if cell_state == CellState::Hidden
                    || ((cell_state == CellState::Flagged || cell_state == CellState::Marked)
                        && all)
                {
                    self.query_update(i, j);
                }
            }
        }
        if original_game_state == GameState::Won {
            // preserve win even if cells were revealed
            self.game_state = GameState::Won;
        }
    }

    pub fn get_flag_locations(&self) -> Vec<(usize, usize)> {
        let mut flag_locations = vec![];
        let (h, w) = self.state.shape();
        for i in 0..h {
            for j in 0..w {
                let cell_state = self.state.get(i, j).unwrap();
                if cell_state == CellState::Flagged {
                    flag_locations.push((i, j));
                }
            }
        }
        flag_locations
    }

    pub fn submit(&mut self) -> Result<GameState, String> {
        if self.game_state != GameState::Running {
            return Err(format!("Game state must be 'Running' to submit, current state is: {:?}", self.game_state));
        }
        let flag_locations = self.get_flag_locations();
        if self.minefield.submit(&flag_locations) {
            self.game_state = GameState::Won;
        } else {
            self.game_state = GameState::Lost;
        }
        Ok(self.game_state)
    }

    pub fn flag(&mut self, row: usize, col: usize) -> CellState {
        let new_state = match self.state.get(row, col).unwrap() {
            CellState::Hidden => CellState::Flagged,
            CellState::Flagged => CellState::Hidden,
            other => other,
        };
        self.state.set(row, col, new_state).unwrap();
        new_state
    }

    pub fn get_state(&self) -> &grid::Grid<CellState> {
        &self.state
    }

    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::field;

    #[test]
    fn scenario_0() {
        let mut client = Client::new_random(10, 10, 0);
        assert_eq!(client.flag(0, 0), CellState::Flagged);
        assert_eq!(client.flag(0, 0), CellState::Hidden);
        assert_eq!(client.query_update(0, 0), Cell::Clean(0));
        assert_eq!(
            client.get_state().get(0, 0).unwrap(),
            CellState::Revealed(Cell::Clean(0))
        );
        assert_eq!(client.query_smart(0, 0), GameState::Running);
        for i in 0..10 {
            for j in 0..10 {
                assert_eq!(
                    client.get_state().get(i, j),
                    Some(CellState::Revealed(Cell::Clean(0)))
                );
            }
        }
    }
    #[test]
    fn scenario_1() {
        let (minefield, bomb_locations) = field::test::generate_test_minefield();
        let mut c = Client::from_minefield(minefield);
        // ensure state is pristine when client is created
        let state = c.get_state();
        let (height, width) = state.shape();
        for i in 0..height {
            for j in 0..width {
                assert_eq!(state.get(i, j), Some(CellState::Hidden));
            }
        }
        // ensure game state is 'Running'
        assert_eq!(c.get_game_state(), GameState::Running);

        for (i, j) in bomb_locations {
            c.flag(i, j);
        }
        assert_eq!(c.submit(), Ok(GameState::Won));
        assert_eq!(c.get_game_state(), GameState::Won);

        // it is not allowed to submit another set of bombs. This should return an Err(_)
        assert!(c.submit().is_err());
    }

    #[test]
    fn reveal() {
        let (minefield, _) = field::test::generate_test_minefield();
        let mut c = Client::from_minefield(minefield);
        // submitting an incorrect set of flagged cell results in losing the game
        assert_eq!(c.submit(), Ok(GameState::Lost));
        assert_eq!(c.get_game_state(), GameState::Lost);

        // mark some cells
        let marked_cells:&[(usize,usize)] = &[(1,1), (2,4)];
        for &(i, j) in marked_cells {
            c.flag(i, j);
        }
        // revealing the grid (all=false) reveals all Hidden cells (not flagged and marked)
        c.reveal(false);
        for (i, j, &val) in c.get_state().iter() {
            if marked_cells.contains(&(i, j)) {
                assert_eq!(val, CellState::Flagged);
            } else {
                assert!(matches!(val, CellState::Revealed(_)));
            }
        }
    }
}
