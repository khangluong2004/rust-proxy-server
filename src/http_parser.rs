use crate::request::Request;
use crate::response::Response;
use std::error::Error;
use std::io::Read;
use std::net::TcpStream;

pub struct HttpParser<'a> {
    stream: &'a mut TcpStream,
    buffer: Vec<u8>,
    pub lines: String,
}

impl<'a> HttpParser<'a> {
    const CRLF_LEN: usize = 2;
    const READ_BUFFER_SIZE: usize = 1024;
    const RESPONSE_MAX_SIZE: usize = 100_000;
    pub fn new(stream: &'a mut TcpStream) -> Self {
        HttpParser {
            stream,
            buffer: Vec::new(),
            lines: String::new(),
        }
    }

    fn read_line(self: &mut HttpParser<'a>) -> Result<String, Box<dyn Error>> {
        loop {
            // check for \r\n
            // Rust uses UTF8, which is backward-compatible with ASCII and ISO-8859-1
            let line = String::from_utf8(self.buffer.clone())?;
            if line.contains("\r\n") {
                let line = line.split("\r\n").nth(0).unwrap().to_string();
                self.buffer = self.buffer[(line.len() + "\r\n".len())..].to_owned();
                return Ok(line);
            }

            let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
            let bytes_read = self.stream.read(&mut buffer)?;
            buffer.resize(bytes_read, 0);
            self.buffer.extend_from_slice(&buffer);
        }
    }

    pub fn read_request(self: &mut HttpParser<'a>) -> Result<Request, Box<dyn Error>> {
        self.lines.clear();

        loop {
            let line = self.read_line()?;
            self.lines += (line.clone() + "\r\n").as_str();

            if line == "" {
                return Request::from_string(self.lines.clone());
            }
        }
    }

    pub fn read_response_header(self: &mut HttpParser<'a>) -> Result<Response, Box<dyn Error>> {
        self.lines.clear();

        loop {
            let line = self.read_line()?;
            self.lines += (line.clone() + "\r\n").as_str();

            if line == "" {
                return Response::from_string(self.lines.clone());
            }
        }
    }

    // Call after reading the response, to add additional header
    // Return the new request
    pub fn add_header(self: &HttpParser<'a>, mut request: String, key: String, value: &String) -> String{
        // Truncate the last 2 "\r\n" bytes
        request.truncate(self.lines.len() - HttpParser::CRLF_LEN);
        // Add the new key value to the header
        let new_header = key + ": " + value + "\r\n";
        request.push_str(&new_header);
        // Add back CRLfF
        request.push_str("\r\n");

        request
    }

    pub fn read_bytes(self: &mut HttpParser<'a>) -> Result<Vec<u8>, Box<dyn Error>> {
        // Read the remaining after reading the header 
        // Note, since buffer is shrunk to its exact length, no need to truncate
        // bunch of zeros
        if self.buffer.len() > 0 {
            self.lines += &String::from_utf8(self.buffer.clone())?;
            let result = self.buffer.to_vec();
            self.buffer.clear();
            return Ok(result);
        }

        let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
        let bytes_read = self.stream.read(&mut buffer)?;
        buffer.resize(bytes_read, 0);
        
        // No need to store if the length exceeds cache requirement.
        // Max size reached would be 101,024 bytes, which is acceptable.
        if self.lines.len() < Self::RESPONSE_MAX_SIZE {
            self.lines += &String::from_utf8(buffer.clone())?;
        }
        Ok(buffer)
    }

    // Task 3: Cache-control parser helper functions

    // Special parse for cache header: Split by comma, and treat quoted string 
    // as 1 token
    // Rules from RFC9110:
    // Without quotation mark: "!" / "#" / "$" / "%" / "&" / "'" / "*"
    //  / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
    //  / DIGIT / ALPHA
    // With quotation mark: Any character, except \" and \\
    // If there is backlash, ignore all rules and treat next char as character
    // Should only see backlash inside quotation mark
    pub fn cache_control_split(self: &HttpParser<'a>, cache_header: &String) -> Vec<String>{
        let mut result = Vec::new();
        let mut cur_str = String::new();
        let mut is_quoted = false;
        let mut is_backlash = false;
        for c in cache_header.chars(){
            if !is_backlash {
                // End the word if is not in quote and get comma
                if !is_quoted && c == ',' {
                    result.push(cur_str.clone());
                    cur_str.clear();
                    continue;
                }

                // Skip space and htab if not in quoted
                if !is_quoted && (c == ' ' || c == '\t'){
                    continue;
                }  

                // Start quote
                if c == '"'{
                    is_quoted = !is_quoted;
                }

                // Turn on backlash if is backlash
                if is_quoted && c == '\\'{
                    is_backlash = true;
                }
            }
            
            // Turn off backlash
            if is_backlash {
                is_backlash = false;
            }

            cur_str.push(c);
        }

        // Add the end, if any
        if cur_str.len() > 0 {
            result.push(cur_str);
        }
        
        result
    }

    // Task 4 helpers: Extract expiry time from directive
    pub fn get_cache_expire(self: &HttpParser<'a>, cache_directive_list: &Vec<String>) -> Option<u32>{
        for cache_directive in cache_directive_list {
            if !cache_directive.contains("max-age=") {
                continue;
            }
            
            let prefix_len = "max-age=".len();
            match cache_directive[prefix_len..].parse::<u32>(){
                Ok(expiry_time) => {return Some(expiry_time);},
                Err(_) => {return None;}
            };
        }

        None
    }

    
}
