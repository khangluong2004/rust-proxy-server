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
    pub const CRLF: &'static str = "\r\n";
    pub const CRLF_BYTES: &'static [u8] = "\r\n".as_bytes();
    pub const CRLF_LEN: usize = Self::CRLF.len();
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

}
