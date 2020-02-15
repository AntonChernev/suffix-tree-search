#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde;

use std::env;
use std::fs::read_to_string;

use routing::router;

mod suffix_tree;
mod routing;
mod handlers;

fn main() {
    let addr = "127.0.0.1:8000";
    println!("Listening for requests at http://{}", addr);

    let mut text_path = env::current_dir().unwrap();
    text_path.push("text/text.txt");
    let text = read_to_string(text_path).unwrap();

    gotham::start(addr, router(&text[..]));
}
