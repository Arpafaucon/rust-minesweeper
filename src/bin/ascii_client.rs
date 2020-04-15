
extern crate minesweeper;

use minesweeper::minefield;
use minesweeper::ascii_client;

fn main() {

    let (height, width) = (12,30);
    let num_bombs = 20;
    let c = minefield::client::Client::new_random(height, width, num_bombs);
    let mut tc = ascii_client::AsciiClient {client: c};
    tc.mainloop();
}