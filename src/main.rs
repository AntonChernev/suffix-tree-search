#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate serde;

use routing::router;

mod suffix_tree;
mod routing;
mod handlers;

fn main() {
    let addr = "127.0.0.1:8000";
    println!("Listening for requests at http://{}", addr);

    gotham::start(addr, router());
    // let suf_tree = SuffixTree::new("abcbcabdccbdbdbcvvabbacccvbbfabaddaba");
    // println!("{}", suf_tree);
}
