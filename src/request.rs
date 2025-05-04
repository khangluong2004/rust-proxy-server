use std::collections::HashMap;
use std::error::Error;

#[derive(Clone)]
pub struct Request {
    // pub method: String,
    pub url: String,
    // pub format: String,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn get_host(self: &Request) -> String {
        self.headers.get("host").unwrap().clone()
    }

    pub fn from_string(request: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();

        // first line is special
        let first = request.split("\r\n").nth(0).unwrap();
        let [_method, url, _format] = &first
            .split(" ")
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>()[..]
        else {
            // TODO: fix this panic
            panic!("invalid header");
        };

        for line in request.split("\r\n").skip(1) {
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

        Ok(Request {
            // method: method.clone(),
            url: url.clone(),
            // format: format.clone(),
            headers,
        })
    }
}