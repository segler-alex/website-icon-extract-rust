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
//! They get analyzed for pixel size and only the necessary bytes are downloaded for that to happen.
//! 
//! # Example
//! ```rust
//! let url = "https://google.com";
//! let list = ImageLink::from_website(url, "TEST", 5).unwrap();
//! println("{:?}", list);
//! ```

use imagesize::blob_size;
use imagesize::image_type;
use imagesize::ImageSize;
pub use imagesize::ImageType;
use log::trace;

use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::RANGE;
use reqwest::header::USER_AGENT;
use reqwest::IntoUrl;
use reqwest::Url;

use quick_xml::events::Event;
use quick_xml::Reader;

/// Holds information about an image
#[derive(Debug)]
pub struct ImageLink {
    /// Url to image
    pub url: Url,
    /// Type of image
    pub image_type: ImageType,
    /// Pixel width of image
    pub width: usize,
    /// Pixel height of image
    pub height: usize,
}

impl ImageLink {
    pub fn new<U: IntoUrl, P: AsRef<str>>(
        url: U,
        user_agent: P,
        tcp_timeout: u64,
    ) -> Result<Self, Box<dyn Error>> {
        let url = url.into_url()?;
        let (image_size, image_type) = get_pixel_size(url.clone(), user_agent, tcp_timeout)?;
        Ok(ImageLink {
            url,
            image_type,
            width: image_size.width,
            height: image_size.height,
        })
    }

    /// Extracts information about icons from website by:
    /// * Download and analyze a html page from http/https url.
    /// * Return all found icon urls.
    /// * Check their sizes by downloading the first 100 bytes
    /// # Arguments
    /// * `url` - An url to check
    /// * `user_agent` - User agent header string for http requests
    /// * `tcp_timeout` - Http timeout in seconds
    /// # Example
    /// ```rust
    /// let url = "https://google.com";
    /// let list = ImageLink::from_website(url, "TEST", 5).unwrap();
    /// println("{:?}", list);
    /// ```
    pub fn from_website<P, Q>(
        base_url: P,
        user_agent: Q,
        tcp_timeout: u64,
    ) -> Result<Vec<ImageLink>, Box<dyn Error>>
    where
        P: AsRef<str>,
        Q: AsRef<str>,
    {
        let base_url = Url::parse(base_url.as_ref())?;
        let response = Client::new()
            .get(base_url.clone())
            .timeout(Duration::new(tcp_timeout, 0))
            .header(USER_AGENT, user_agent.as_ref())
            .send()?;
    
        let mut list: Vec<String> = analyze_location(response)?;
        list.push(String::from("/favicon.ico"));
        Ok(list
            .iter()
            .filter_map(|unfiltered_url| base_url.join(&unfiltered_url).ok())
            .filter_map(|image_url| ImageLink::new(image_url, user_agent.as_ref(), tcp_timeout).ok())
            .collect())
    }
}

/// Search html content for links to icons and return them
fn analyze_content(content: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut reader = Reader::from_str(content);
    reader.trim_text(true);
    reader.check_end_names(false);
    let mut buf = Vec::new();
    let mut list: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
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

/// Download part of the file and try to load as image.
/// If possible return pixel dimensions (x,y)
fn get_pixel_size<U: IntoUrl, P: AsRef<str>>(
    url: U,
    user_agent: P,
    tcp_timeout: u64,
) -> Result<(ImageSize, ImageType), Box<dyn Error>> {
    let url = url.into_url()?;
    let response = Client::new()
        .get(url.clone())
        .timeout(Duration::new(tcp_timeout, 0))
        .header(RANGE, "bytes=0-99")
        .header(USER_AGENT, user_agent.as_ref())
        .send()?;
    let data: Vec<u8> = response.bytes()?.to_vec();
    let pixel_size = blob_size(&data)?;
    let image_type = image_type(&data)?;
    trace!(
        "{}, downloaded bytes: {}, pixels: {}x{}, type: {:?}",
        url,
        data.len(),
        pixel_size.width,
        pixel_size.height,
        image_type
    );
    Ok((pixel_size, image_type))
}

/// Download the file and analyze the content
/// Try to extract links to images.
/// # Returns
/// List of image urls
fn analyze_location(response: Response) -> Result<Vec<String>, Box<dyn Error>> {
    let content_type = response.headers().get(CONTENT_TYPE);
    if let Some(content_type) = content_type {
        if content_type.to_str().unwrap_or("").starts_with("text/html") {
            let content = response.text()?;
            let list = analyze_content(&content)?;
            return Ok(list);
        }
    }
    Ok(Vec::new())
}

fn attr_to_hash(
    reader: &quick_xml::Reader<&[u8]>,
    e: quick_xml::events::attributes::Attributes,
) -> HashMap<String, String> {
    let attrs_hashed: HashMap<String, String> = e
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
        .map(|x| {
            (
                reader.decoder().decode(x.key.local_name().as_ref()).map(|b| b.to_string().to_lowercase()),
                reader.decoder().decode(&x.value).map(|c| c.to_string()),
            )
        })
        .filter(|i| i.0.is_ok() && i.1.is_ok())
        .map(|j| (j.0.unwrap(), j.1.unwrap()))
        .collect();
    attrs_hashed
}

fn extract(
    attrs_hashed: &HashMap<String, String>,
    names: &Vec<String>,
    key_name: &str,
    content: &str,
) -> Vec<String> {
    let mut list: Vec<String> = vec![];
    let name: Option<&String> = attrs_hashed.get(key_name);
    let content = attrs_hashed.get(content);
    if let Some(name) = name {
        if let Some(content) = content {
            let name: String = name.to_lowercase();
            let content = content.to_lowercase();
            if names.contains(&name) {
                list.push(content.to_string());
            }
        }
    }
    list
}

/// Check a single html element if it does contain a link to a describing image
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
    let mut list: Vec<String> = Vec::new();

    match e.name().local_name().as_ref() {
        b"meta" => {
            let attrs_hashed = attr_to_hash(&reader, e.attributes());
            let l = extract(&attrs_hashed, &meta_name_attrs, "name", "content");
            list.extend(l);
            let l = extract(&attrs_hashed, &meta_property_attrs, "property", "content");
            list.extend(l);
        }
        b"link" => {
            let attrs_hashed = attr_to_hash(&reader, e.attributes());
            let l = extract(&attrs_hashed, &link_rel_attrs, "rel", "href");
            list.extend(l);
        }
        _ => {}
    };

    list
}
