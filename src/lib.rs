//! This library connects to a given url and tries to find references to icons that represent the page.
//! Multiple standards are used for this:
//! * default favicon.ico in root and as "link rel"
//!   [wikipedia.org/wiki/Favicon](https://en.wikipedia.org/wiki/Favicon)
//! * apple touch icon
//!   [Apple docs](https://developer.apple.com/library/content/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html)
//! * Open graph image
//!   [ogp.me](http://ogp.me/)
//! * Windows 8 tile images
//!   [Microsoft technet](https://technet.microsoft.com/en-us/windows/dn255024(v=vs.60)#msapplication-TileImage)
//! 
//! All images are converted to absolute urls and checked if connecting to them works.
//! It does a GET request, but closes the connection after the http header.
extern crate native_tls;
extern crate quick_xml;
extern crate url;

mod request;

use url::Url;

use request::Request;
use std::collections::HashMap;
use std::error::Error;

use quick_xml::Reader;
use quick_xml::events::Event;

/// Download and analyze a html page from http/https url.
/// Return all found icon urls.
/// # Arguments
/// * `url` - An url to check
/// * `user_agent` - User agent header string for http requests
/// * `tcp_timeout` - Http timeout in seconds
pub fn extract_icons(url: &str, user_agent: &str, tcp_timeout: u32) -> Result<Vec<String>, Box<Error>> {
    let x = Request::new(url, user_agent, tcp_timeout)?;
    let mut list: Vec<String> = analyze_location(x)?;
    list.push(String::from("/favicon.ico"));
    let list_filtered: Vec<String> = list.iter()
        .map(|x| normalize_url(url, x))
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .filter(|x| check_connection(x, user_agent, tcp_timeout))
        .collect();
    Ok(list_filtered)
}

fn check_connection(url: &str, user_agent: &str, tcp_timeout: u32) -> bool {
    let r = Request::new(url, user_agent, tcp_timeout);
    match r {
        Ok(r) => {
            let code = r.get_code();
            if code == 200 {
                return true;
            }
            return false;
        }
        _ => {
            return false;
        }
    }
}

fn normalize_url(base_url_str: &str, url: &str) -> Result<String, Box<Error>> {
    let base_url = Url::parse(base_url_str)?;
    let abs_url = base_url.join(&url)?;
    return Ok(abs_url.to_string());
}

fn analyze_location(mut x: Request) -> Result<Vec<String>, Box<Error>> {
    let content_type = x.get_header("content-type");
    if let Some(content_type) = content_type {
        if content_type.starts_with("text/html") {
            x.read_content()?;
            let list = analyze_content(x.get_content())?;
            return Ok(list);
        }
    }
    Ok(Vec::new())
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
    attrs_hashed: &HashMap<String, String>,
    names: &Vec<String>,
    name: &str,
    content: &str,
) -> Vec<String> {
    let mut list: Vec<String> = vec![];
    let name: Option<&String> = attrs_hashed.get(name);
    let content = attrs_hashed.get(content);
    if name.is_some() && content.is_some() {
        let name: String = name.unwrap().to_lowercase();
        let content = content.unwrap().to_lowercase();
        if names.contains(&name) {
            list.push(content.to_string());
        }
    }
    list
}

fn check_start_elem(
    reader: &quick_xml::Reader<&[u8]>,
    e: &quick_xml::events::BytesStart<'_>,
) -> Vec<String> {
    let meta_name_attrs: Vec<String> = vec![
        String::from("msapplication-TileImage"),
        String::from("msapplication-square70x70logo"),
        String::from("msapplication-square150x150logo"),
        String::from("msapplication-square310x310logo"),
        String::from("msapplication-wide310x150logo"),
    ];
    let meta_property_attrs: Vec<String> = vec![String::from("og:image")];
    let link_rel_attrs: Vec<String> = vec![
        String::from("apple-touch-icon"),
        String::from("shortcut icon"),
        String::from("icon"),
    ];
    let item: String = reader.decode(e.name()).to_string().to_lowercase();
    let mut list: Vec<String> = Vec::new();

    if item == "meta" {
        let attrs_hashed = attr_to_hash(&reader, e.attributes());
        let l = extract(&attrs_hashed, &meta_name_attrs, "name", "content");
        list.extend(l);

        let l = extract(&attrs_hashed, &meta_property_attrs, "property", "content");
        list.extend(l);
    }
    if item == "link" {
        let attrs_hashed = attr_to_hash(&reader, e.attributes());
        let l = extract(&attrs_hashed, &link_rel_attrs, "rel", "href");
        list.extend(l);
    }
    list
}

fn analyze_content(content: &str) -> Result<Vec<String>, Box<Error>> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    reader.check_end_names(false);
    let mut buf = Vec::new();
    let mut list: Vec<String> = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                list.extend(check_start_elem(&reader, e));
            }
            Ok(Event::Start(ref e)) => {
                list.extend(check_start_elem(&reader, e));
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
