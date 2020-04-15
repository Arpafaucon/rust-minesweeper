extern crate minesweeper;

use minesweeper::minefield;
use minesweeper::tui_client;




fn main() {

    let (height, width) = (12,30);
    let num_bombs = 20;
    let c = minefield::client::Client::new_random(height, width, num_bombs);
    // let mut ac = ascii_client::AsciiClient {client:c};
    let mut tc = tui_client::TuiClient::new(c);
    tc.mainloop();

    // println!("{}", ac.client.minefield);
    // println!("{}", ac);
    // // ac.client.flag(0, 0);
    // // println!("{}", ac);
    // ac.client.query_smart(0, 4);
    // println!("{}", ac);

    // for i in 0..height {
    //     for j in 0..width {
    //         ac.client.query_smart(i, j);
    //     }
    // }
    // println!("{}", ac);
}

/*
extern crate termion;

use termion::{clear, color, cursor, style};

use std::io::{stdin, stdout, Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
fn main() {
    // Get the standard input stream.
    let stdin = stdin();
    // Get the standard output stream and go to raw mode.
    let mut stdout = MouseTerminal::from(stdout()).into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}q to exit. Type stuff, use alt, and so on.{}",
        // Clear the screen.
        termion::clear::All,
        // Goto (1,1).
        termion::cursor::Goto(1, 1),
        // Hide the cursor.
        termion::cursor::Hide
    )
    .unwrap();
    // Flush stdout (i.e. make the output appear).
    stdout.flush().unwrap();

    for c in stdin.events() {
        // Clear the current line.
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine
        )
        .unwrap();

        // Print the key we type...
        match c.unwrap() {
            // Exit.
            Event::Mouse(me) => match me {
                MouseEvent::Press(_, a, b) | MouseEvent::Release(a, b) | MouseEvent::Hold(a, b) => {
                    write!(stdout, "{}<-", cursor::Goto(a, b)).unwrap();
                }
            },
            Event::Key(Key::Char('q')) => break,
            Event::Key(Key::Char(c)) => println!("{}", c),
            Event::Key(Key::Alt(c)) => println!("Alt-{}", c),
            Event::Key(Key::Ctrl(c)) => println!("Ctrl-{}", c),
            Event::Key(Key::Left) => println!("<left>"),
            Event::Key(Key::Right) => println!("<right>"),
            Event::Key(Key::Up) => println!("<up>"),
            Event::Key(Key::Down) => println!("<down>"),
            _ => println!("Other"),
        }

        // Flush again.
        stdout.flush().unwrap();
    }

    // Show the cursor again before we exit.
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
*/