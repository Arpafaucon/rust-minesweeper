mod minefield;
mod ascii_client;

fn main() {

    let (height, width) = (8,12);
    let num_bombs = 1;
    let mut c = minefield::client::Client::new(height, width, num_bombs);
    let mut ac = ascii_client::AsciiClient {client:c};
    ac.mainloop();
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
