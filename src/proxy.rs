use crate::http_parser::HttpParser;
use crate::request::Request;
use std::collections::{HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};

pub struct Proxy {
    does_cache: bool,
    cache: Vec<(String, String, Request)>,
}

impl Proxy {
    pub fn new(does_cache: bool) -> Self {
        Self {
            does_cache,
            cache: Vec::new(),
        }
    }

    fn get_cached(self: &mut Proxy, request: &String) -> Option<String> {
        for (i, entry) in self.cache.iter().enumerate() {
            if &entry.0 == request {
                // place lru to back of list
                let entry = entry.clone();
                self.cache.remove(i);
                self.cache.push(entry.clone());
                return Some(entry.1);
            }
        }

        None
    }

    fn add_cache(self: &mut Proxy, req: String, res: String, request: Request) {
        self.cache.push((req, res, request));
    }

    fn evict_lru(self: &mut Proxy) {
        if self.cache.len() > 0 {
            let last_request = &self.cache[self.cache.len() - 1].2;
            println!(
                "Evicting {} {} from cache",
                last_request.get_host(),
                last_request.url
            );

            self.cache.remove(0);
        }
    }

    fn handle_connection(self: &mut Proxy, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        // set socket params
        stream.set_nodelay(true)?;
        println!("Accepted");

        // get request
        let mut parser = HttpParser::new(&mut stream);
        let request = parser.read_request()?;

        // magic number 3
        let lines = parser.lines.split("\r\n").collect::<Vec<&str>>();
        println!("Request tail {}", lines[lines.len() - 3]);

        let request_lines = parser.lines;
        let host = request.get_host();

        if self.does_cache && request_lines.len() < 2000 {
            // check cache
            if let Some(value) = self.get_cached(&request_lines) {
                // use cache
                println!("Serving {} {} from cache", host, request.url);

                stream.write_all(value.as_bytes())?;
                stream.shutdown(Shutdown::Both)?;
                return Ok(());
            } else {
                // evict
                if self.cache.len() == 10 {
                    self.evict_lru();
                }
            }
        }

        println!("GETting {} {}", host, request.url);

        // create remote server socket and forward request
        let mut proxy = TcpStream::connect(format!("{}:80", host))?;
        proxy.write(request_lines.as_bytes())?;

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

        let response_lines = parser.lines;
        if self.does_cache && request_lines.len() < 2000 && response_lines.len() < 100_000 {
            // cache response
            self.add_cache(request_lines, response_lines, request);
        }

        Ok(())
    }

    pub fn start_server(self: &mut Proxy, port: u16) -> Result<(), Box<dyn Error>> {
        // start listener
        // note that the default backlog is 128 in rust, and it cannot be changed
        let listener = TcpListener::bind(format!(":::{}", port))?;
        for stream in listener.incoming() {
            let result = stream
                .map_err(|err| Box::new(err) as Box<dyn Error>)
                .and_then(|stream| self.handle_connection(stream));
            match result {
                Ok(()) => {}
                Err(err) => {
                    println!("error: {}", err);
                } // ignored errors
            }
        }

        Ok(())
    }
}
