use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Response {
    pub status_code: String,
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn from_string(response: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();
        let mut first_line = true;
        let mut status_code: Option<String> = None;
        // first line is special
        for line in response.split("\r\n") {
            // First line is special
            if first_line {
                let [_version, local_status_code, _status_msg] = &line
                    .split(" ")
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<String>>()[..]
                else {
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Can't find status code")));
                };
                status_code = Some(local_status_code.clone());
                first_line = false;
                continue;
            }


            if line == "" {
                break;
            }

            // parse header
            if let [header, value] = line.split(": ").collect::<Vec<&str>>()[..] {
                headers.insert(header.to_string().to_lowercase(), value.to_string());
            } else {
                println!("skipping unknown header {}", line);
            }
        }

        if let Some(status_code_val) = status_code {
            return Ok(Response {headers, status_code: status_code_val });
        };
        
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Error in status code parsing")));
        
    }
}
