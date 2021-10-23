use ::website_icon_extract::ImageLink;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("needs 1 parameter");
        std::process::exit(1);
    }
    let url: String = args.nth(1).expect("needs 1 parameter");
    let result = ImageLink::from_website(url, "agent", 10);

    match result {
        Ok(result) => {
            for item in result {
                println!("{:?}", item);

            }
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
