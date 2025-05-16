use crate::http_parser::HttpParser;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Response {
    pub status_code: String,
    pub headers: HashMap<String, String>,
}

impl Response {
    const RESPONSE_STATUS_CODE_POSITION: usize = 1;
    const HEADER_PARTS: usize = 2;
    pub fn from_string(response: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();
        let mut first_line = true;
        let mut status_code: Option<String> = None;
        // first line is special
        for line in response.split(HttpParser::CRLF) {
            // First line is special
            if first_line {
                let parts: Vec<&str> = line.split(" ").collect();
                if Self::RESPONSE_STATUS_CODE_POSITION >= parts.len() {
                    return Err("can't find status code".into());
                }
                status_code = Some(
                    parts[Self::RESPONSE_STATUS_CODE_POSITION]
                        .to_string()
                        .clone(),
                );
                first_line = false;
                continue;
            }

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

        let status_code = status_code.ok_or("error parsing status code")?;
        Ok(Response {
            headers,
            status_code,
        })
    }
}
