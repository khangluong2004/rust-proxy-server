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
    const READ_BUFFER_SIZE: usize = 1024;
    pub fn new(stream: &'a mut TcpStream) -> Self {
        HttpParser {
            stream,
            buffer: Vec::new(),
            lines: String::new(),
        }
    }

    fn read_line(self: &mut HttpParser<'a>) -> Result<String, Box<dyn Error>> {
        loop {
            loop {
                // check for \r\n
                let line = String::from_utf8(self.buffer.clone())?;
                if line.contains("\r\n") {
                    let line = line.split("\r\n").nth(0).unwrap().to_string();
                    self.buffer = self.buffer[line.len() + "\r\n".len()..].to_owned();

                    return Ok(line);
                } else {
                    break;
                }
            }

            let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
            self.stream.read(&mut buffer)?;
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

    pub fn read_bytes(self: &mut HttpParser<'a>) -> Result<Vec<u8>, Box<dyn Error>> {
        if self.buffer.len() > 0 {
            let result = self.buffer.to_vec();
            self.buffer.clear();
            return Ok(result);
        }

        let mut buffer = vec![0; Self::READ_BUFFER_SIZE];
        self.stream.read(&mut buffer)?;
        Ok(buffer.to_vec())
    }
}
