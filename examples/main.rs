extern crate website_icon_extract;
extern crate clap;
extern crate native_tls;

use std::env;

fn main() {
    match env::args().nth(1) {
        Some(url) => {
            website_icon_extract::test(&url);
        }
        None => {
            println!("1 parameter needed");
        }
    };
}
