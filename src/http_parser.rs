use crate::request::Request;
use crate::response::Response;
use std::error::Error;
use std::io::Read;
use std::net::TcpStream;

// Http stream parser
pub struct HttpParser<'a> {
    stream: &'a mut TcpStream,
    // buffer for the currently read but unhandled bytes
    buffer: Vec<u8>,
    // data for the entire request/response
    data: Vec<u8>,
    // header length for both request/response
    header_length: usize,
}

impl<'a> HttpParser<'a> {
    const CRLF: &'static str = "\r\n";
    const CRLF_BYTES: &'static [u8] = "\r\n".as_bytes();
    const CRLF_LEN: usize = Self::CRLF.len();
    const READ_BUFFER_SIZE: usize = 1024;
    const RESPONSE_MAX_SIZE: usize = 100_000;

    pub fn new(stream: &'a mut TcpStream) -> Self {
        HttpParser {
            stream,
            buffer: Vec::new(),
            data: Vec::new(),
            header_length: 0,
        }
    }

    // Return the header lines in utf-8
    pub fn header_lines(self: &HttpParser<'a>) -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8(
            self.data[..self.header_length].to_owned(),
        )?)
    }

    // Returns parser data
    pub fn data(self: &HttpParser<'a>) -> Vec<u8> {
        self.data.clone()
    }

    // Read a single line ended by \r\n, return the bytes as is
    fn read_line(self: &mut HttpParser<'a>) -> Result<Vec<u8>, Box<dyn Error>> {
        loop {
            // check for \r\n
            if let Some(index) = self.buffer.windows(2).position(|w| w == Self::CRLF_BYTES) {
                let line = self.buffer[..index + Self::CRLF_LEN].to_owned();
                self.buffer = self.buffer[index + Self::CRLF_LEN..].to_owned();
                return Ok(line);
            }

            let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
            let bytes_read = self.stream.read(&mut buffer)?;
            buffer.resize(bytes_read, 0);
            self.buffer.extend_from_slice(&buffer);
        }
    }

    // Read a http request from the stream
    pub fn read_request(self: &mut HttpParser<'a>) -> Result<Request, Box<dyn Error>> {
        self.data.clear();

        loop {
            let line = self.read_line()?;
            self.data.extend_from_slice(&line);

            let line = String::from_utf8(line)?;
            if line == "\r\n" {
                self.header_length = self.data.len();
                return Request::from_string(String::from_utf8(self.data.clone())?);
            }
        }
    }

    // Read an http response from the stream, will also consume the blank line
    pub fn read_response_header(self: &mut HttpParser<'a>) -> Result<Response, Box<dyn Error>> {
        self.data.clear();

        loop {
            let line = self.read_line()?;
            self.data.extend_from_slice(&line);

            let line = String::from_utf8(line)?;
            if line == "\r\n" {
                self.header_length = self.data.len();
                return Response::from_string(String::from_utf8(self.data.clone())?);
            }
        }
    }

    // Read a series of bytes
    pub fn read_bytes(self: &mut HttpParser<'a>) -> Result<Vec<u8>, Box<dyn Error>> {
        // Read the remaining after reading the header
        // Note, since buffer is shrunk to its exact length, no need to truncate
        // bunch of zeros
        if self.buffer.len() > 0 {
            self.data.extend_from_slice(&self.buffer);
            let result = self.buffer.to_vec();
            self.buffer.clear();
            return Ok(result);
        }

        let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
        let bytes_read = self.stream.read(&mut buffer)?;
        buffer.resize(bytes_read, 0);

        // No need to store if the length exceeds cache requirement.
        // Max size reached would be 101,024 bytes, which is acceptable.
        if self.data.len() < Self::RESPONSE_MAX_SIZE {
            self.data.extend_from_slice(&buffer);
        }
        Ok(buffer)
    }

    // Appends the header key value pair to a header_lines that ends with the \r\n
    pub fn append_header(header_lines: String, key: &String, value: &String) -> String {
        let stripped = header_lines[..header_lines.len() - Self::CRLF_LEN].to_owned();
        format!("{}{}{}: {}{}", stripped, Self::CRLF, key, value, Self::CRLF)
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
    fn cache_control_split(cache_header: &String) -> Vec<String> {
        let mut result = vec![];
        let mut current = "".to_string();

        let mut ptr = 0;
        let mut is_quoted = false;
        let text = cache_header.as_bytes();
        while ptr < text.len() {
            let c = char::from(text[ptr]);
            let c_str = c.to_string();
            match (is_quoted, c) {
                (true, '"') => {
                    is_quoted = false;
                    current += &c_str;
                }
                (true, '\\') => {
                    ptr += 1;
                    current += &text[ptr].to_string();
                }
                (false, '"') => {
                    is_quoted = true;
                    current += &c_str;
                }
                (false, ',') => {
                    result.push(current);
                    current = "".to_string();
                }
                _ => {
                    current += &c_str;
                }
            }

            ptr += 1;
        }

        if !current.is_empty() {
            result.push(current);
        }

        result
    }
    
    pub fn parse_cache_control 

    // Task 4 helpers: Extract expiry time from directive
    pub fn get_cache_expire(cache_directive_list: &Vec<String>) -> Option<u32> {
        for cache_directive in cache_directive_list {
            if !cache_directive.contains("max-age=") {
                continue;
            }

            let prefix_len = "max-age=".len();
            match cache_directive[prefix_len..].parse::<u32>() {
                Ok(expiry_time) => {
                    return Some(expiry_time);
                }
                Err(_) => {
                    return None;
                }
            };
        }

        None
    }
}
