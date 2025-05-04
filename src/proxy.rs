use crate::http_parser::HttpParser;
use crate::request::Request;
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};

use std::time::{self, Duration, Instant};

pub struct Proxy {
    does_cache: bool,
    cache: Vec<(String, String, Request, Instant, Option<u32>)>,
}

impl Proxy {
    const TAIL_OFFSET: usize = 3;
    const CACHE_MAX: usize = 10;
    pub fn new(does_cache: bool) -> Self {
        Self {
            does_cache,
            cache: Vec::new(),
        }
    }

    // Task 3:

    fn is_cache_allowed(self: &mut Proxy, cache_header: &String) -> bool{
        !(cache_header == "private" 
            || cache_header == "no-store"
            || cache_header == "no-cache"
            || cache_header == "max-age=0"
            || cache_header == "must-validate"
            || cache_header == "proxy-revalidate")
    } 

    // Task 4 helpers
    fn get_cache_expire(self: &Proxy, cache_header: &String) -> Option<u32>{
        if !cache_header.contains("max-age=") {
            return None;
        }
        
        let prefix_len = "max-age".len();
        match cache_header[prefix_len..].parse::<u32>(){
            Ok(expiry_time) => Some(expiry_time),
            Err(_) => None
        }
    }

    fn check_time_out(self: &Proxy, time_now: Instant, expiry: Option<u32>) -> bool{
        let Some(expiry_secs) = expiry else {
            return false;
        };

        let elapsed_secs = time_now.elapsed().as_secs();

        if elapsed_secs > (expiry_secs as u64) {
            return false;
        }

        true
    }

    // Other tasks

    fn get_cached(self: &mut Proxy, request: &String) -> Option<String> {
        let found_entry_index = self.cache.iter().position(|entry| &entry.0 == request);
        let Some(index) = found_entry_index else {
            return None;
        };
        
        let entry_ref = self.cache.get(index)?;
        let host = &entry_ref.2.get_host();
        let url = &entry_ref.2.url;

        if self.check_time_out(entry_ref.3, entry_ref.4){
            println!("Stale entry for {} {}", host, url);
            self.cache.remove(index);
            return None;
        }

        let entry_copy = self.cache.get(index)?.clone();
        let result = entry_copy.1.clone();
        self.cache.remove(index);
        self.cache.push(entry_copy);
        Some(result)

    }

    fn add_cache(self: &mut Proxy, req: String, res: String, request: Request, expiry: Option<u32>) {
        let time_now = Instant::now();
        self.cache.push((req, res, request, time_now, expiry));
    }

    fn evict_lru(self: &mut Proxy) {
        if self.cache.len() > 0 {
            let last_request = &self.cache[0].2;
            println!(
                "Evicting {} {} from cache",
                last_request.get_host(),
                last_request.url
            );

            self.cache.remove(0);
        }
    }

    fn handle_connection(self: &mut Proxy, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        // set socket params (Why?)
        // No need for SO_REUSEADDR as set by default
        stream.set_nodelay(true)?;
        println!("Accepted");

        // get request
        let mut parser = HttpParser::new(&mut stream);
        let request = parser.read_request()?;

        let lines = parser.lines.split("\r\n").collect::<Vec<&str>>();
        println!("Request tail {}", lines[lines.len() - Self::TAIL_OFFSET]);

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
            }     
        }

        println!("GETting {} {}", host, request.url);

        // create remote server socket and forward request
        let mut proxy = TcpStream::connect(format!("{}:80", host))?;
        proxy.write(request_lines.as_bytes())?;

        // read server header
        let mut parser = HttpParser::new(&mut proxy);
        let response = parser.read_response_header()?;
        
        // Get content length
        let content_length = response
            .headers
            .get("content-length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()?;
        println!("Response body length {}", content_length);


        // Get cache-control
        let mut allow_cache = true;
        let mut expiry_time = None;
        if let Some(cache_control_val) = response
            .headers
            .get("cache-control") {
                allow_cache = self.is_cache_allowed(cache_control_val);
                expiry_time = self.get_cache_expire(cache_control_val);
        };

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
            if !allow_cache {
                println!("Not caching {} {}", host, request.url);
            } else {
                // evict
                if self.cache.len() == Self::CACHE_MAX {
                    self.evict_lru();
                }
                
                // cache response
                self.add_cache(request_lines, response_lines, request, expiry_time);
            }
        }

        // Close the server connection as well
        proxy.shutdown(Shutdown::Both)?;
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
