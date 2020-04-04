use crate::minefield;
use minefield::client::{CellState, Client, GameState};
use minefield::field::Cell;
use std::io;
use std::str;

pub struct AsciiClient {
    pub client: minefield::client::Client,
}

fn to_char_mono(c: &CellState) -> String {
    match c {
        CellState::Hidden => String::from("█"),
        CellState::Flagged => String::from("▓"),
        CellState::Revealed(r) => match r {
            Cell::Bomb => String::from("*"),
            Cell::Clean(0) => String::from(" "),
            Cell::Clean(i) => format!("{}", i),
        },
        CellState::Marked => String::from("▟"),
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Command {
    None,
    Exit,
    Query(usize, usize),
    Flag(usize, usize),
    Submit,
}

impl AsciiClient {
    fn check_coordinates(&self, c: Command) -> Command {
        match c {
            Command::Query(i, j) | Command::Flag(i, j) => {
                if let Some(_) = self.client.get_state().index(i, j) {
                    // valid coordinates
                    c
                } else {
                    Command::None
                }
            }
            other => other
        }
    }

    fn parse_coordinate(input: &str) -> Option<(usize, usize)> {
        let mut chars = input.chars();
        let col_char = chars.next().unwrap();
        if !col_char.is_ascii_alphabetic() {
            return None;
        }
        let col = col_char.to_digit(36).unwrap() as usize - 10;
        let row_char = chars.next().unwrap();
        if !row_char.is_digit(10) {
            return None;
        }
        let row = row_char.to_digit(10).unwrap() as usize;
        Some((row, col))
    }

    fn parse_input(input: &str) -> Command {
        let low_i = input.to_ascii_lowercase();
        match &low_i[..] {
            _ if !low_i.is_ascii() => Command::None,
            "q" => Command::Exit,
            "s" => Command::Submit,
            _ if low_i.len() == 2 => match Self::parse_coordinate(&low_i) {
                Some((row, col)) => Command::Query(row, col),
                None => Command::None
            },
            _ if low_i.len() == 3 => {
                if let Some((row, col)) = Self::parse_coordinate(&low_i[1..]) {
                    match &low_i[..1] {
                        "f" => Command::Flag(row, col),
                        "d" => Command::Query(row, col),
                        _ => Command::None
                    }
                } else {
                    Command::None
                }
            }
            _ => Command::None,
        }
    }

    pub fn mainloop(&mut self) {
        let mut input = String::new();
        let mut current_command: Command;
        while self.client.get_game_state() == GameState::Running {
            // display board
            println!("{}", self);
            // read line
            current_command = Command::None;
            while current_command == Command::None {
                println!("Enter a command: 'xY' or 'dxY' to dig, 'Q' to exit, 'fXY' to flag. E.g.: 'da3'.");
                input.clear();
                match io::stdin().read_line(&mut input) {
                    Ok(n) => {
                        // println!("{}", input);
                        current_command = Self::parse_input(input.trim());
                    }
                    Err(error) => println!("error: {}", error),
                }
                current_command = self.check_coordinates(current_command)
            }
            match current_command {
                Command::Exit => {
                    break;
                }
                Command::Query(row, col) => {
                    self.client.query_smart(row, col);
                }
                Command::Submit => {
                    self.client.submit().unwrap();
                },
                Command::Flag(row, col) => {
                    self.client.flag(row, col);
                }
                _ => (),
            }
        }
        // println!("Game finished: {:?}", self.client.get_game_state());
        self.client.reveal(true);
        println!("{}", self);

    }
}

impl std::fmt::Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", to_char_mono(self))
    }
}

impl std::fmt::Display for AsciiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alphabet = "abcdefghijklmnopqrstuvwxyz";
        let state = self.client.get_state();
        writeln!(f, "state: {:?}", self.client.get_game_state())?;
        let (h, w) = self.client.get_state().shape();
        write!(f, "   {} \n", &alphabet[..w])?;
        write!(f, "  ┏{}┓\n", "━".repeat(w))?;
        for i in 0..h {
            write!(f, "{:2}┃", i)?;
            for j in 0..w {
                write!(f, "{}", state.get(i, j).unwrap())?;
            }
            write!(f, "┃\n")?;
        }
        write!(f, "  ┗{}┛\n", "━".repeat(w))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn input() {
        assert_eq!(AsciiClient::parse_input("a1"), Command::Query(1, 0));
        assert_eq!(AsciiClient::parse_input("e8"), Command::Query(8, 4));
        assert_eq!(AsciiClient::parse_input("q"), Command::Exit);
        assert_eq!(AsciiClient::parse_input("s"), Command::Submit);
        assert_eq!(AsciiClient::parse_input("nn"), Command::None);
        assert_eq!(AsciiClient::parse_input("55"), Command::None);
        assert_eq!(AsciiClient::parse_input("fa4"), Command::Flag(4, 0));
        assert_eq!(AsciiClient::parse_input("dz4"), Command::Query(4, 25));

    }
}
