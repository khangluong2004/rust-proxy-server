use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
}

impl Response {
    pub fn from_string(response: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();

        // first line is special
        for line in response.split("\r\n").skip(1) {
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

        Ok(Response { headers })
    }
}
