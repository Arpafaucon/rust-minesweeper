extern crate minesweeper;

use minesweeper::minefield;
use minesweeper::tui_client;


fn main() {
    let (height, width) = (12,30);
    let num_bombs = 20;
    let c = minefield::client::Client::new_random(height, width, num_bombs);
    let mut tc = tui_client::TuiClient::new(c);
    tc.mainloop();
}