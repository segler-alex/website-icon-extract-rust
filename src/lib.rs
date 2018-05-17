extern crate native_tls;
extern crate quick_xml;
extern crate url;

mod request;

use request::Request;
use std::collections::HashMap;
use std::error::Error;

use quick_xml::Reader;
use quick_xml::events::Event;

pub fn test(url: &str) -> Result<Vec<String>, Box<Error>> {
    let x = Request::new(url, "TEST", 5)?;
    analyze_location(x)?;
    Ok(Vec::new())
}

fn analyze_location(mut x: Request) -> Result<(), Box<Error>> {
    let content_type = x.get_header("content-type");
    if let Some(content_type) = content_type {
        if content_type.starts_with("text/html") {
            x.read_content();
            analyze_content(x.get_content())?;
        }
    }
    Ok(())
}

fn attr_to_hash(
    reader: &quick_xml::Reader<&[u8]>,
    e: quick_xml::events::attributes::Attributes,
) -> HashMap<String, String> {
    let attrs_hashed: HashMap<String, String> = e.filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .map(|x| {
            (
                reader.decode(x.key).to_string().to_lowercase(),
                reader.decode(&x.value).to_string(),
            )
        })
        .collect();
    attrs_hashed
}

fn extract(
    attrs_hashed: HashMap<String, String>,
    names: &Vec<&str>,
    name: &str,
    content: &str,
) -> Vec<String> {
    let mut list: Vec<String> = vec![];
    let name = attrs_hashed.get(name);
    let content = attrs_hashed.get(content);
    if name.is_some() && content.is_some() {
        let name: &str = name.unwrap();
        let content = content.unwrap();
        if names.contains(&name) {
            list.push(content.to_string());
        }
    }
    list
}

fn analyze_content(content: &str) -> Result<Vec<String>, Box<Error>> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    reader.check_end_names(false);
    let mut buf = Vec::new();
    let mut list: Vec<String> = Vec::new();
    let meta_attrs: Vec<&str> = vec![
        "msapplication-TileImage",
        "msapplication-square70x70logo",
        "msapplication-square150x150logo",
        "msapplication-square310x310logo",
        "msapplication-wide310x150logo",
    ];
    let link_attrs: Vec<&str> = vec!["shortcut icon", "icon"];
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                let item = reader.decode(e.name()).to_string().to_lowercase();
                if item == "meta" {
                    let attrs_hashed = attr_to_hash(&reader, e.attributes());
                    let l = extract(attrs_hashed, &link_attrs, "name", "content");
                    list.extend(l);
                }
                if item == "link" {
                    let attrs_hashed = attr_to_hash(&reader, e.attributes());
                    let l = extract(attrs_hashed, &meta_attrs, "rel", "href");
                    list.extend(l);
                }
            }
            Ok(Event::Start(ref e)) => {
                let item = reader.decode(e.name()).to_string().to_lowercase();
                if item == "meta" {
                    let attrs_hashed = attr_to_hash(&reader, e.attributes());
                    let l = extract(attrs_hashed, &link_attrs, "name", "content");
                    list.extend(l);
                }
                if item == "link" {
                    let attrs_hashed = attr_to_hash(&reader, e.attributes());
                    let l = extract(attrs_hashed, &meta_attrs, "rel", "href");
                    list.extend(l);
                }
            }
            Ok(Event::End(_)) => {}
            Ok(Event::Text(_)) => {}
            Ok(Event::Eof) => break,
            Err(e) => {
                println!("Error at position {}: {:?}", reader.buffer_position(), e);
                //break;
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
        buf.clear();
    }
    Ok(list)
}
