use crate::cache::{Cache, CacheRecord};
use crate::headers;
use crate::headers::CacheControlHeader;
use crate::http_parser::HttpParser;
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
    const RESPONSE_CACHE_LENGTH: usize = 100 * 1024;
    const NOT_MODIFIED_STATUS_CODE: &'static str = "304";

    pub fn new(does_cache: bool) -> Self {
        Self {
            does_cache,
            cache: Cache::new(),
        }
    }

    fn handle_connection(self: &mut Proxy, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
        // No need for SO_REUSEADDR as set by default
        stream.set_nodelay(true)?;
        println!("Accepted");

        // get request
        let mut request_parser = HttpParser::new(&mut stream);
        let request = request_parser.read_request()?;

        // need to keep the original for cache indexing
        let mut request_headers = request_parser.header_lines()?;
        let original_request_headers = request_parser.header_lines()?;

        let lines = request_headers
            .split(HttpParser::CRLF)
            .collect::<Vec<&str>>();

        // If length less than 3 (TAIL_OFFSET), error
        println!("Request tail {}", lines.get(lines.len() - Self::TAIL_OFFSET)
            .ok_or("Unexpected format for request headers")?);

        let request_host = request.get_host()?;
        // Already throw if can't get url
        let request_url = request.url.clone();
        let mut is_expired = false;
        let mut option_cache_record: Option<CacheRecord> = None;

        if self.does_cache && request_headers.len() < Self::REQUEST_CACHE_LENGTH {
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
                    request_headers = headers::append_header(
                        request_headers,
                        &(headers::IF_MODIFIED_SINCE_HEADER.into()),
                        &cache_value.date,
                    );
                }

                option_cache_record = Some(cache_value);
            } else {
                // evict if full, task 2
                if self.cache.is_full() {
                    let record = self.cache.remove_lru_cache()?;
                    println!(
                        "Evicting {} {} from cache",
                        record.request.get_host()?,
                        record.request.url
                    );
                }
            }
        }

        println!("GETting {} {}", request_host, request_url);

        // create remote server socket and forward request
        let mut proxy = TcpStream::connect(format!("{}:80", request_host))?;
        proxy.write(&request_headers.as_bytes())?;

        // read server header
        let mut response_parser = HttpParser::new(&mut proxy);
        let response = response_parser.read_response_header()?;

        // Get status code for task 5. If 304, return early.
        if self.does_cache && response.status_code == Self::NOT_MODIFIED_STATUS_CODE {
            if let Some(cache_value) = option_cache_record {
                // use cache and log
                println!("Serving {} {} from cache", request_host, request.url);
                stream.write_all(&cache_value.response)?;

                if is_expired {
                    println!("Entry for {} {} unmodified", request_host, request.url);
                }

                stream.shutdown(Shutdown::Both)?;
                proxy.shutdown(Shutdown::Both)?;

                return Ok(());
            }
        }

        // Otherwise, proxy and cache (if applicable)
        // Get content length
        let content_length = response
            .headers
            .get(headers::CONTENT_LENGTH_HEADER)
            .ok_or("expected a content length in the response")?
            .parse::<usize>()?;
        println!("Response body length {}", content_length);

        // Get cache-control
        let mut allow_cache = true;
        let mut expiry_time = None;
        if let Some(cache_control_val) = response.headers.get(headers::CACHE_CONTROL_HEADER) {
            let cache_control = CacheControlHeader::new(cache_control_val)?;
            allow_cache = cache_control.should_cache();
            if allow_cache {
                expiry_time = cache_control.cache_expire();
            }
        };

        // Get date
        let date = response
            .headers
            .get(headers::DATE_HEADER)
            .ok_or::<Box<dyn Error>>("no date in response".into())?
            .clone();

        // forward header
        stream.write_all(&response_parser.data())?;

        // read and forward server response body
        let mut count = 0;
        while count < content_length {
            let bytes = response_parser.read_bytes(Self::RESPONSE_CACHE_LENGTH)?;
            stream.write_all(&bytes)?;
            count += bytes.len();
        }
        stream.shutdown(Shutdown::Both)?;

        let evict_if_expired = |proxy: &mut Self| -> Result<(), Box<dyn Error>> {
            if is_expired {
                let record = proxy.cache.remove_cache(&original_request_headers)?;
                println!(
                    "Evicting {} {} from cache",
                    record.request.get_host()?,
                    record.request.url
                );
            }

            Ok(())
        };

        let response_data = response_parser.data();
        if self.does_cache
            && request_headers.len() < Self::REQUEST_CACHE_LENGTH
            && response_data.len() <= Self::RESPONSE_CACHE_LENGTH
        {
            if !allow_cache {
                println!("Not caching {} {}", request_host, request_url);
                evict_if_expired(self)?;
            } else {
                // cache response
                // Add cache will overwrite the old response,
                // and add_lru will flip entries to the end if exist.
                // So no need to evict (specs also don't allow log here)
                self.cache.add_cache(
                    original_request_headers,
                    request,
                    response_data,
                    expiry_time,
                    date,
                )?;
            }
        } else {
            evict_if_expired(self)?;
        }

        // Close the server connection as well
        proxy.shutdown(Shutdown::Both)?;
        Ok(())
    }

    pub fn start_server(self: &mut Proxy, port: u16) -> Result<(), Box<dyn Error>> {
        // start listener
        // note that the default backlog is 128 in rust, and it cannot be changed
        let listener = TcpListener::bind(format!("[::]:{}", port))?;
        for stream in listener.incoming() {
            let result = stream
                .map_err(|e| e.into())
                .and_then(|stream| self.handle_connection(stream));
            match result {
                Ok(()) => {}
                Err(err) => {
                    println!("handle_connection error: {}", err);
                } // ignored errors
            }
        }

        Ok(())
    }
}
