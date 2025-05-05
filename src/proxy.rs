use crate::http_parser::HttpParser;
use crate::cache::Cache;
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};


pub struct Proxy {
    does_cache: bool,
    cache: Cache,
}

impl Proxy {
    const TAIL_OFFSET: usize = 3;
    pub fn new(does_cache: bool) -> Self {
        Self {
            does_cache,
            cache: Cache::new(),
        }
    }

    // Task 3:
    fn is_cache_allowed_single(self: &Proxy, cache_header: &String) -> bool{
        // TODO: Is "max-age=\"0\"" valid
        !(cache_header == "private" 
            || cache_header == "no-store"
            || cache_header == "no-cache"
            || cache_header == "max-age=0"
            || cache_header == "must-validate"
            || cache_header == "proxy-revalidate")
    }

    // Special parse for cache header: Split by comma, and treat quoted string 
    // as 1 token
    fn cache_control_split(self: &Proxy, cache_header: &String) -> Vec<String>{
        let mut result = Vec::new();
        let mut cur_str = String::new();
        let mut is_quoted = false;
        for c in cache_header.chars(){
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

            cur_str.push(c);
        }

        // Add the end, if any
        if cur_str.len() > 0 {
            result.push(cur_str);
        }
        
        result
    }

    fn is_cache_allowed(self: &Proxy, word_list: &Vec<String>) -> bool{
        for word in word_list{
            if !self.is_cache_allowed_single(word) {
                return false;
            }
        }

        true
    }

    // Task 4 helpers
    fn get_cache_expire(self: &Proxy, cache_directive_list: &Vec<String>) -> Option<u32>{
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

        let original_request_lines = parser.lines.clone();
        let mut request_lines = parser.lines.clone();
        let host = request.get_host();
        let url = request.url.clone();
        let mut is_expired = false;

        if self.does_cache && request_lines.len() < 2000 {
            // check cache
            if let Some((option_string, local_is_expired)) = self.cache.get_cached(&request_lines) {
                is_expired = local_is_expired;

                if let Some(cache_value) = option_string {
                    if !is_expired {
                        // use cache
                        println!("Serving {} {} from cache", host, request.url);
                        stream.write_all(cache_value.response.as_bytes())?;
                        stream.shutdown(Shutdown::Both)?;
                        return Ok(());
                    } else {
                        // Logging for task 4
                        println!("Stale entry for {} {}", host, url);
                        // Modify the request_lines for task 5
                        request_lines = parser.add_header(request_lines, String::from("If-Modified-Since"), &cache_value.date);
                    }
                    
                }
            }

        }

        println!("GETting {} {}", host, url);

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
                let word_list = self.cache_control_split(cache_control_val);
                allow_cache = self.is_cache_allowed(&word_list);
                if allow_cache {
                    expiry_time = self.get_cache_expire(&word_list);
                }
        };

        // Get date
        let Some(date_ref) = response.headers.get("date")
        else {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, "No date in request")));
        };
        let date = date_ref.clone();

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

        // If is_expired, remove from cache and load back
        if is_expired {
            self.cache.remove_cache(&request_lines);
        }

        if self.does_cache && request_lines.len() < 2000 && response_lines.len() < 100_000 {
            if !allow_cache {
                println!("Not caching {} {}", host, url);
                
                // Cacheable, but not allowed to cache
                if is_expired {
                    println!("Evicting {} {} from cache", host, url);
                }
            } else {
                // cache response
                let is_evicted = self.cache.add_cache(original_request_lines, response_lines, expiry_time, date);
                if is_evicted {
                    println!("Evicting {} {} from cache", host, url);
                }
            }
        } else {
            // If expired, and can't be cached, log evict
            // Already removed while trying to get from cache
            if is_expired {
                println!("Evicting {} {} from cache", host, url);
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
