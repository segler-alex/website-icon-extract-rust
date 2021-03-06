extern crate website_icon_extract;
extern crate native_tls;

use std::env;

fn main() {
    match env::args().nth(1) {
        Some(url) => {
            let list = website_icon_extract::extract_icons(&url, "TEST", 5);
            match list {
                Ok(o) => {
                    println!("list: {:?}", o);
                }
                Err(e) => {
                    println!("empty list: {}", e);
                }
            }
            
        }
        None => {
            println!("1 parameter needed");
        }
    };
}
