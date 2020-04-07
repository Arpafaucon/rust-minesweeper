use crate::minefield;
extern crate termion;

use minefield::client::{CellState, GameState};
use minefield::field::Cell;
use std::io;
use std::str;

use termion::{clear, color, cursor, style};

use std::io::{Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

const GRID_OFFSET: (u16,u16) = (3, 3);

pub struct TuiClient {
    current_cursor: (usize, usize),
    client: minefield::client::Client,
}

impl TuiClient {
    fn valid_cursor(&self) -> () {}

    pub fn new(client: minefield::client::Client) -> TuiClient {
        TuiClient {
            current_cursor: (1, 1),
            client: client,
        }
    }

    pub fn mainloop(&self) {
        // Get the standard input stream.
        let stdin = std::io::stdin();
        // Get the standard output stream and go to raw mode.
        let mut stdout = MouseTerminal::from(std::io::stdout()).into_raw_mode().unwrap();
        let mut cursor = (4u16, 4u16);

        write!(
            stdout,
            "{}{}q to exit. Type stuff, use alt, and so on",
            // Clear the screen.
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        ).unwrap();
        stdout.flush().unwrap();
        for c in stdin.events() {
            match c.unwrap() {
                // Exit
                Event::Key(Key::Char('q')) => break,
                //
                Event::Key(Key::Left) => cursor.0 -= 1,
                Event::Key(Key::Right) => cursor.0 += 1,
                Event::Key(Key::Up) => cursor.1 -= 1,
                Event::Key(Key::Down) => cursor.1 += 1,
                _ => {}
            }
            write!(
                stdout,
                "{clear}{pos}row={row}, col={col}",
                clear= clear::CurrentLine,
                pos = cursor::Goto(1,1),
                row = cursor.1,
                col = cursor.0
            )
            .unwrap();
            write!(
                stdout,
                "{pos}{blink}",
                pos = cursor::Goto(cursor.0, cursor.1),
                blink = cursor::BlinkingBlock
            )
            .unwrap();
            stdout.flush().unwrap();
        }
    }
}