use crate::http_parser::HttpParser;
use crate::cache::{CacheRecord, Cache};
use std::error::Error;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};


pub struct Proxy {
    does_cache: bool,
    cache: Cache,
}

impl Proxy {
    const TAIL_OFFSET: usize = 3;
    const REQUEST_CACHE_LENGTH: usize = 2000;
    const RESPONSE_CACHE_LENGTH: usize = 100_000;
    const IF_MODIFIED_SINCE_HEADER: &'static str = "If-Modified-Since";
    const CONTENT_LENGTH_HEADER: &'static str = "content-length";

    pub fn new(does_cache: bool) -> Self {
        Self {
            does_cache,
            cache: Cache::new(),
        }
    }

    fn handle_connection(self: &mut Proxy, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        // TODO: Check set socket params (Why?)
        // No need for SO_REUSEADDR as set by default
        stream.set_nodelay(true)?;
        println!("Accepted");

        // get request
        let mut request_parser = HttpParser::new(&mut stream);
        let request = request_parser.read_request()?;
        let mut request_headers = request_parser.header_lines()?;
        let request_data = request_parser.data();
        let lines = request_headers.split("\r\n").collect::<Vec<&str>>();
        println!("Request tail {}", lines[lines.len() - Self::TAIL_OFFSET]);


        let request_host = request.get_host();
        let request_url = request.url.clone();
        let mut is_expired = false;
        let mut option_cache_record: Option<CacheRecord> = None;

        if self.does_cache && request_data.len() < Self::REQUEST_CACHE_LENGTH {
            // check cache
            if let Some((cache_value, local_is_expired)) = self.cache.get(&request_headers) {
                is_expired = local_is_expired;

                if !is_expired {
                    // use cache
                    println!("Serving {} {} from cache", request_host, request_url);
                    stream.write_all(&cache_value.response)?;
                    stream.shutdown(Shutdown::Both)?;
                    return Ok(());
                } else {
                    // Logging for task 4
                    println!("Stale entry for {} {}", request_host, request_url);
                    // Modify the request_lines for task 5
                    request_headers = HttpParser::append_header(request_headers, &(Self::IF_MODIFIED_SINCE_HEADER.into()), &cache_value.date);
                }

                option_cache_record = Some(cache_value);
            }
        }

        println!("GETting {} {}", request_host, request_url);

        // create remote server socket and forward request
        let mut proxy = TcpStream::connect(format!("{}:80", request_host))?;
        proxy.write(&request_data)?;

        // read server header
        let mut response_parser = HttpParser::new(&mut proxy);
        let response = response_parser.read_response_header()?;

        // Get status code for task 5. If 304, return early.
        if self.does_cache && response.status_code == "304" {
            if let Some(cache_value) = option_cache_record {
                // use cache and log
                println!("Serving {} {} from cache", request_host, request.url);
                stream.write_all(&cache_value.response)?;

                if is_expired {
                    println!("Entry for {} {} unmodified", request_host, request.url);
                }

                stream.shutdown(Shutdown::Both)?;

                return Ok(());
            }
        }

        // Otherwise, proxy and cache (if applicable)
        // Get content length
        let content_length = response
            .headers
            .get(Self::CONTENT_LENGTH_HEADER)
            .ok_or("expected a content length in the response")?
            .parse::<usize>()?;
        println!("Response body length {}", content_length);

        // Get cache-control
        let mut allow_cache = true;
        let mut expiry_time = None;
        if let Some(cache_control_val) = response
            .headers
            .get("cache-control") {
                let word_list = HttpParser::cache_control_split(cache_control_val);
                let allow_cache_local = self.cache.is_cache_allowed(&word_list);
                allow_cache = allow_cache_local;
                if allow_cache {
                    expiry_time = HttpParser::get_cache_expire(&word_list);
                }
        };

        // Get date
        let date = response.headers.get("date").ok_or::<Box<dyn Error>>("no date in response".into())?.clone();

        // forward header
        stream.write_all(&response_parser.data())?;

        // read and forward server response body
        let mut count = 0;
        while count < content_length {
            let bytes = response_parser.read_bytes()?;
            stream.write_all(&bytes)?;
            count += bytes.len();
        }
        stream.shutdown(Shutdown::Both)?;

        // // If is_expired, remove from cache and load back
        // // Handle logging later on (to follow specs sequence)
        // if is_expired {
        //     self.cache.remove_cache(&request_headers);
        // }

        let response_data = response_parser.data();
        if self.does_cache && request_headers.len() < Self::REQUEST_CACHE_LENGTH && response_data.len() < Self::RESPONSE_CACHE_LENGTH {
            if !allow_cache {
                println!("Not caching {} {}", request_host, request_url);
                if is_expired {
                    let record = self.cache.remove_cache(&request_headers);
                    println!("Evicting {} {} from cache", record.request.get_host(), record.request.url);
                }
                
            } else {
                if is_expired {
                    let record = self.cache.remove_cache(&request_headers);
                    println!("Evicting {} {} from cache", record.request.get_host(), record.request.url);
                }
                
                // cache response
                let record = self.cache.add_cache(request_headers.clone(), request, response_data, expiry_time, date);
                if let Some(record) = record {
                    println!("Evicting {} {} from cache", record.request.get_host(), record.request.url);
                }
            }
        }
        // else {
        //     // If expired, and can't be cached, log evict
        //     if is_expired {
        //         println!("Evicting {} {} from cache", host, url);
        //     }
        // }

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
