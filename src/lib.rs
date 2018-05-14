extern crate native_tls;
extern crate url;
extern crate quick_xml;

mod request;

use request::Request;
use std::error::Error;
use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

pub fn test(url: &str) -> Result<Vec<String>,Box<Error>> {
    let x = Request::new(url, "TEST", 5)?;
    analyze_location(x)?;
    Ok(Vec::new())
}

fn analyze_location(mut x: Request) -> Result<(),Box<Error>> {
    let content_type = x.get_header("content-type");
    if let Some(content_type) = content_type {
        if content_type.starts_with("text/html") {
            x.read_content();
            analyze_content(x.get_content())?;
        }
    }
    Ok(())
}

fn decode_attribute(name_field: &str, content_field: &str) -> Option<String> {
    None
}

fn analyze_content(mut content: &str) -> Result<(),Box<Error>> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                let item = reader.decode(e.name()).to_string().to_lowercase();
                if item == "meta"{
                    let attrs_hashed: HashMap<String,String> = e
                        .attributes()
                        .filter(|x| x.is_ok())
                        .map(|x| x.unwrap())
                        .map(|x| (reader.decode(x.key).to_string().to_lowercase(),reader.decode(&x.value).to_string()))
                        .collect();
                    
                    let name = attrs_hashed.get("name");
                    let content = attrs_hashed.get("content");
                    if name.is_some() && content.is_some() {
                        let name = name.unwrap();
                        let content = content.unwrap();

                        if (name == "msapplication-TileImage"){
                        }else if (name == "msapplication-square70x70logo"){
                        }else if (name == "msapplication-square150x150logo"){
                        }else if (name == "msapplication-square310x310logo"){
                        }else if (name == "msapplication-wide310x150logo"){
                        }
                    }
                }
                if item == "link"{
                    let attrs_hashed: HashMap<String,String> = e
                        .attributes()
                        .filter(|x| x.is_ok())
                        .map(|x| x.unwrap())
                        .map(|x| (reader.decode(x.key).to_string().to_lowercase(),reader.decode(&x.value).to_string()))
                        .collect();
                    
                    let name = attrs_hashed.get("rel");
                    let content = attrs_hashed.get("href");
                    if name.is_some() && content.is_some() {
                        let name = name.unwrap();
                        let content = content.unwrap();

                        if (name == "shortcut icon"){
                        }else if (name == "icon"){
                        }
                    }
                }
            }
            Ok(Event::Start(ref e)) => {

            }
            Ok(Event::End(_)) => {

            }
            Ok(Event::Text(e)) => {

            }
            Ok(Event::Eof) => break,
            Err(e) => {
                println!("Error at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
        buf.clear();
    }
    Ok(())
}