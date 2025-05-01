use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::string::ParseError;
use std::time::Duration;

struct CliArguments {
    port: u16,
    does_cache: bool,
}

#[derive(Debug)]
struct Request {
    method: String,
    url: String,
    format: String,
    headers: HashMap<String, String>,
}

impl Request {
    fn from_string(request: String) -> Result<Self, Box<dyn Error>> {
        let mut headers = HashMap::new();

        // first line is special
        let first = request.split("\r\n").nth(0).unwrap();
        let [method, url, format] = &first
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
            method: method.clone(),
            url: url.clone(),
            format: format.clone(),
            headers,
        })
    }
}

#[derive(Debug)]
struct Response {
    pub headers: HashMap<String, String>,
}

impl Response {
    fn from_string(response: String) -> Result<Self, Box<dyn Error>> {
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

struct HttpParser<'a> {
    stream: &'a mut TcpStream,
    buffer: Vec<u8>,
    lines: String,
}

impl<'a> HttpParser<'a> {
    fn new(stream: &'a mut TcpStream) -> Self {
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

            let mut buffer = vec![0; 1024];
            self.stream.read(&mut buffer)?;
            self.buffer.extend_from_slice(&buffer);
        }
    }

    fn read_request(self: &mut HttpParser<'a>) -> Result<Request, Box<dyn Error>> {
        self.lines.clear();

        loop {
            let line = self.read_line()?;
            self.lines += (line.clone() + "\r\n").as_str();

            if line == "" {
                return Request::from_string(self.lines.clone());
            }
        }
    }

    fn read_response_header(self: &mut HttpParser<'a>) -> Result<Response, Box<dyn Error>> {
        self.lines.clear();

        loop {
            let line = self.read_line()?;
            self.lines += (line.clone() + "\r\n").as_str();

            if line == "" {
                return Response::from_string(self.lines.clone());
            }
        }
    }

    fn read_bytes(self: &mut HttpParser<'a>) -> Result<Vec<u8>, Box<dyn Error>> {
        if self.buffer.len() > 0 {
            let result = self.buffer.to_vec();
            self.buffer.clear();
            return Ok(result);
        }

        let mut buffer = vec![0; 1024];
        self.stream.read(&mut buffer)?;
        Ok(buffer.to_vec())
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // set socket params
    stream.set_nodelay(true)?;
    println!("Accepted");

    // get request
    let mut parser = HttpParser::new(&mut stream);
    let request = parser.read_request()?;
    println!("[debug] request {:?}", request);
    let lines = parser.lines.split("\r\n").collect::<Vec<&str>>();
    // magic number 3
    println!("Request tail {}", lines[lines.len() - 3]);

    // create remote server socket and forward request
    let host = request.headers.get("host").ok_or("no host header")?;
    println!("GETting {} {}", host, request.url);
    let mut proxy = TcpStream::connect(format!("{}:80", host))?;
    proxy.write(parser.lines.as_bytes())?;

    // read server header
    let mut parser = HttpParser::new(&mut proxy);
    let response = parser.read_response_header()?;
    let content_length = response
        .headers
        .get("content-length")
        .unwrap_or(&"0".to_string())
        .parse::<usize>()?;
    println!("Response body length {}", content_length);

    // forward header
    stream.write_all(parser.lines.as_bytes())?;

    // read and forward server response
    let mut count = 0;
    while count < content_length {
        let bytes = parser.read_bytes()?;
        stream.write_all(&bytes)?;
        count += bytes.len();
    }

    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn start_server(args: CliArguments) -> Result<(), Box<dyn Error>> {
    // start listener
    // note that the default backlog is 128 in rust, and it cannot be changed
    let listener = TcpListener::bind(format!(":::{}", args.port))?;
    for stream in listener.incoming() {
        let result = stream
            .map_err(|err| Box::new(err) as Box<dyn Error>)
            .and_then(handle_connection);
        match result {
            Ok(()) => {}
            Err(err) => {
                println!("error: {}", err);
            } // ignored errors
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut port = 0u16;
    let mut does_cache = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                port = args[i + 1].parse::<u16>()?;
                i += 2;
            }
            "-c" => {
                does_cache = true;
                i += 1;
            }
            _ => {
                panic!("unknown argument {}", args[i]);
            }
        }
    }

    start_server(CliArguments { does_cache, port })
}
