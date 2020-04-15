use crate::minefield;
extern crate termion;

use minefield::client::{CellState, GameState};
use minefield::field::Cell;
use std::convert::TryInto;
use std::io::Write;
use std::string::ToString;
use std::time::Instant;
use std::usize;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor};

const GRID_OFFSET: (u16, u16) = (5, 3); // (row, col)
const BOX_CHARS: [char; 10] = ['+', '-', '+', '|', '━', '┃', '┏', '┓', '┗', '┛'];

pub struct TuiClient {
    current_cursor: (u16, u16),
    client: minefield::client::Client,
    start_time: Option<Instant>,
}

#[derive(PartialEq, Debug)]
enum TuiAction {
    Exit,
    Flag,
    Query,
    Submit,
    None,
}

pub fn cellstate_to_prettystring(c: &CellState) -> String {
    match c {
        CellState::Hidden => format!(
            "{bg} {bgr}",
            bg = color::Bg(color::White),
            bgr = color::Bg(color::Reset),
        ),
        CellState::Flagged => format!(
            "{fg}{bg}¶{bgr}{fgr}",
            bg = color::Bg(color::White),
            fg = color::Fg(color::Black),
            bgr = color::Bg(color::Reset),
            fgr = color::Fg(color::Reset)
        ),
        CellState::Revealed(r) => match r {
            Cell::Bomb => String::from("*"),
            Cell::Clean(0) => String::from(" "),
            Cell::Clean(i) => format!(
                "{col}{num}{reset}",
                num = i,
                reset = color::Fg(color::Reset),
                col = match i {
                    1 => format!("{}", color::Fg(color::Blue)),
                    2 => format!("{}", color::Fg(color::Green)),
                    3 => format!("{}", color::Fg(color::Red)),
                    4 => format!("{}", color::Fg(color::Cyan)),
                    5 => format!("{}", color::Fg(color::LightGreen)),
                    6 => format!("{}", color::Fg(color::Magenta)),
                    _ => format!("{}", color::Fg(color::White)),
                }
            ),
        },
        CellState::Marked => String::from("▟"),
    }
}

impl TuiClient {
    fn to_grid_coordinates(&self, cursor_row: u16, cursor_col: u16) -> Option<(usize, usize)> {
        let (grid_h, grid_w) = self.client.get_state().shape();
        let admissible_rows = (GRID_OFFSET.0 + 1)..(GRID_OFFSET.0 + 1 + grid_h as u16);
        let admissible_cols = (GRID_OFFSET.1 + 1)..(GRID_OFFSET.1 + 1 + grid_w as u16);
        if admissible_rows.contains(&cursor_row) && admissible_cols.contains(&cursor_col) {
            let grid_row: usize = (cursor_row - admissible_rows.start) as usize;
            let grid_col: usize = (cursor_col - admissible_cols.start) as usize;
            Some((grid_row, grid_col))
        } else {
            None
        }
    }

    pub fn new(client: minefield::client::Client) -> TuiClient {
        TuiClient {
            current_cursor: (GRID_OFFSET.1 + 1, GRID_OFFSET.0 + 1),
            client: client,
            start_time: None,
        }
    }

    fn draw_box<T>(
        stdout: &mut T,
        origin_x: u16,
        origin_y: u16,
        height: u16,
        width: u16,
    ) -> Result<(), std::io::Error>
    where
        T: std::io::Write,
    {
        assert!(height >= 2 && width >= 2);
        let h_line_width: usize = (width - 2).try_into().unwrap();
        write!(
            stdout,
            "{pos}{ul_corner}{u_line}{ur_corner}",
            pos = cursor::Goto(origin_y, origin_x),
            ul_corner = BOX_CHARS[6],
            ur_corner = BOX_CHARS[7],
            u_line = BOX_CHARS[4].to_string().repeat(h_line_width)
        )?;
        for x in 1..(height - 1) {
            write!(
                stdout,
                "{l_pos}{v_line}{r_pos}{v_line}",
                l_pos = cursor::Goto(origin_y, origin_x + x),
                v_line = BOX_CHARS[5],
                r_pos = cursor::Goto(origin_y + width - 1, origin_x + x),
            )?;
        }
        write!(
            stdout,
            "{pos}{ll_corner}{l_line}{lr_corner}",
            pos = cursor::Goto(origin_y, origin_x + height - 1),
            ll_corner = BOX_CHARS[8],
            lr_corner = BOX_CHARS[9],
            l_line = BOX_CHARS[4].to_string().repeat(h_line_width)
        )
    }

    fn draw<T>(&self, stdout: &mut T) -> Result<(), std::io::Error>
    where
        T: std::io::Write,
    {
        let (grid_h, grid_w) = self.client.get_state().shape();
        let num_flags = self.client.get_flag_locations().len();
        let num_bombs = self.client.num_bombs();
        let cursor_pos = self.current_cursor;
        // clear screen
        write!(stdout, "{}", clear::All)?;
        // write header
        write!(
            stdout,
            "{}{}{} *  MINESWEEPER ¶ {}{}\r\n",
            cursor::Goto(1, 1),
            color::Bg(color::White),
            color::Fg(color::Black),
            color::Bg(color::Reset),
            color::Fg(color::Reset)
        )?;
        let helper_message = if num_bombs == num_flags {
            "(Submit with 's')"
        } else {
            ""
        };
        write!(
            stdout,
            "Flagged bombs: {}/{} {}\r\n",
            num_flags, num_bombs, helper_message
        )?;
        if let Some(start_time) = self.start_time {
            let duration = Instant::now() - start_time;
            write!(stdout, "Time: {:?}", duration)?;
        }
        // write grid borders
        let box_h: u16 = (grid_h + 2).try_into().unwrap();
        let box_w: u16 = (grid_w + 2).try_into().unwrap();
        Self::draw_box(stdout, GRID_OFFSET.0, GRID_OFFSET.1, box_h, box_w)?;
        // write grid
        let state = self.client.get_state();
        for i in 0..grid_h {
            let first_cell_x = GRID_OFFSET.0 + 1 + (i as u16);
            write!(
                stdout,
                "{pos}",
                pos = cursor::Goto(GRID_OFFSET.1 + 1, first_cell_x)
            )?;
            for j in 0..grid_w {
                let cell_state = state.get(i, j).unwrap();
                write!(stdout, "{}", cellstate_to_prettystring(&cell_state))?;
            }
        }
        // put cursor to right position
        write!(
            stdout,
            "{pos}{blink}",
            pos = cursor::Goto(cursor_pos.0, cursor_pos.1),
            blink = cursor::BlinkingBlock
        )?;
        // flush
        stdout.flush()
    }

    fn parse_event(&mut self, c: Event) -> TuiAction {
        let mut target_cursor = self.current_cursor.clone();
        let action = match c {
            // actions
            Event::Key(Key::Char('q')) => TuiAction::Exit,
            Event::Key(Key::Char(' ')) => TuiAction::Query,
            Event::Key(Key::Char('f')) => TuiAction::Flag,
            Event::Key(Key::Char('s')) => TuiAction::Submit,

            // move cursor with keys
            Event::Key(Key::Left) => {
                target_cursor.0 -= 1;
                TuiAction::None
            }
            Event::Key(Key::Right) => {
                target_cursor.0 += 1;
                TuiAction::None
            }
            Event::Key(Key::Up) => {
                target_cursor.1 -= 1;
                TuiAction::None
            }
            Event::Key(Key::Down) => {
                target_cursor.1 += 1;
                TuiAction::None
            }

            Event::Mouse(MouseEvent::Release(i, j)) => {
                target_cursor = (i, j);
                TuiAction::None
            }

            _ => TuiAction::None,
        };

        // check validity of cursor, update state

        if self.to_grid_coordinates(target_cursor.1, target_cursor.0) != None {
            self.current_cursor = target_cursor;
        }
        action
    }
    fn start_timer(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now())
        }
    }

    pub fn mainloop(&mut self) {
        // Get the standard input stream.
        let stdin = std::io::stdin();
        let mut request_exit = false;
        // Get the standard output stream and go to raw mode.
        let mut stdout = MouseTerminal::from(std::io::stdout())
            .into_raw_mode()
            .unwrap();

        self.draw(&mut stdout).unwrap();
        for c in stdin.events() {
            match self.parse_event(c.unwrap()) {
                TuiAction::Flag => {
                    let grid_pos = self
                        .to_grid_coordinates(self.current_cursor.1, self.current_cursor.0)
                        .unwrap();
                    self.client.flag(grid_pos.0, grid_pos.1);
                    self.start_timer();
                }
                TuiAction::Query => {
                    let grid_pos = self
                        .to_grid_coordinates(self.current_cursor.1, self.current_cursor.0)
                        .unwrap();
                    self.client.query_smart(grid_pos.0, grid_pos.1);
                    self.start_timer();
                }
                TuiAction::Exit => {
                    request_exit = true;
                }
                TuiAction::Submit => {
                    self.client.submit().unwrap();
                }
                TuiAction::None => (),
            }
            self.draw(&mut stdout).unwrap();
            if self.client.get_game_state() != GameState::Running || request_exit {
                self.client.reveal(true);
                self.draw(&mut stdout).unwrap();
                break;
            }
        }
        let next_free_line = self.client.get_state().shape().0 as u16 + GRID_OFFSET.0 + 3;
        let goodbye_sentence = match self.client.get_game_state() {
            GameState::Lost if request_exit => "No time anymore ? See you soon !",
            GameState::Lost => "Another time !",
            GameState::Won => "Congratulations ! You're the best! ",
            _ => "How do you do that ????",
        };
        write!(
            stdout,
            "{pos}{clear}{goodbye_sentence}\r\n",
            // Clear the screen.
            pos = cursor::Goto(1, next_free_line),
            clear = clear::AfterCursor,
            goodbye_sentence = goodbye_sentence
        )
        .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use minefield::field::test::generate_test_minefield;

    #[test]
    fn coordinates() {
        let (field, _bomb_locations) = generate_test_minefield();
        let (h, w) = field.shape();
        let client = minefield::client::Client::from_minefield(field);
        let mut t_client = TuiClient::new(client);

        // basic conversion checks
        assert_eq!(
            t_client.to_grid_coordinates(GRID_OFFSET.0 + 1, GRID_OFFSET.1 + 1),
            Some((0, 0))
        );
        assert_eq!(
            t_client.to_grid_coordinates(GRID_OFFSET.0 + (h as u16), GRID_OFFSET.1 + (w as u16)),
            Some((2, 4))
        );
        assert_eq!(
            t_client
                .to_grid_coordinates(GRID_OFFSET.0 + 1 + (h as u16), GRID_OFFSET.1 + (w as u16)),
            None
        );
        assert_eq!(
            t_client
                .to_grid_coordinates(GRID_OFFSET.0 + (h as u16), GRID_OFFSET.1 + 1 + (w as u16)),
            None
        );
        assert_eq!(
            t_client.to_grid_coordinates(GRID_OFFSET.0, GRID_OFFSET.1),
            None
        );

        // Checks that the event parsing method does not allow
        // the cursor to leave the grid

        // Upper left corner
        let upp_left_cursor = t_client.current_cursor;
        assert_eq!(t_client.parse_event(Event::Key(Key::Left)), TuiAction::None);
        assert_eq!(t_client.current_cursor, upp_left_cursor); // cannot go left

        assert_eq!(t_client.parse_event(Event::Key(Key::Up)), TuiAction::None);
        assert_eq!(t_client.current_cursor, upp_left_cursor); // cannot go up

        assert_eq!(t_client.parse_event(Event::Key(Key::Down)), TuiAction::None);
        assert_eq!(
            t_client.current_cursor,
            (upp_left_cursor.0, upp_left_cursor.1 + 1)
        ); // down is allowed

        // Bottom right corner
        let bott_right_curser = (GRID_OFFSET.1 + (w as u16), GRID_OFFSET.0 + (h as u16));
        assert_eq!(
            t_client.parse_event(Event::Mouse(MouseEvent::Release(
                bott_right_curser.0,
                bott_right_curser.1
            ))),
            TuiAction::None
        ); // goto corner
        assert_eq!(
            t_client.parse_event(Event::Key(Key::Right)),
            TuiAction::None
        );
        assert_eq!(t_client.current_cursor, bott_right_curser); // cannot go up
        assert_eq!(t_client.parse_event(Event::Key(Key::Down)), TuiAction::None);
        assert_eq!(t_client.current_cursor, bott_right_curser); // cannot go up
        assert_eq!(t_client.parse_event(Event::Key(Key::Left)), TuiAction::None);
        assert_eq!(
            t_client.current_cursor,
            (bott_right_curser.0 - 1, bott_right_curser.1)
        ); // right is allowed
    }
}
