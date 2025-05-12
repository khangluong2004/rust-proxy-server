use crate::http_parser::HttpParser;
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone)]
pub struct Request {
    pub url: String,
    pub headers: HashMap<String, String>,
}

impl Request {
    const HEADER_PARTS: usize = 2;

    pub fn get_host(self: &Request) -> String {
        self.headers.get("host").unwrap().clone()
    }

    pub fn from_string(request: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();

        // first line is special
        let first = request
            .split(HttpParser::CRLF)
            .nth(0)
            .ok_or("error in parsing request first line")?;
        let [_method, url, _format] = &first
            .split(" ")
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>()[..]
        else {
            return Err("error in parsing request first line".into());
        };

        for line in request.split(HttpParser::CRLF).skip(1) {
            if line == "" {
                break;
            }

            // parse header
            if let [header, value] =
                line.splitn(Self::HEADER_PARTS, ": ").collect::<Vec<&str>>()[..]
            {
                headers.insert(header.to_string().to_lowercase(), value.to_string());
            } else {
                return Err(format!("unknown header {}", line).into());
            }
        }

        Ok(Request {
            url: url.clone(),
            headers,
        })
    }
}
