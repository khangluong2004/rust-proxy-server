use crate::http_parser::HttpParser;
use std::error::Error;

pub const IF_MODIFIED_SINCE_HEADER: &'static str = "If-Modified-Since";
pub const CONTENT_LENGTH_HEADER: &'static str = "content-length";
pub const CACHE_CONTROL_HEADER: &'static str = "cache-control";
pub const DATE_HEADER: &'static str = "date";
pub const DATE_HEADER_DEFAULT: &'static str = "Wed, 21 May 2025 01:01:56 GMT";
const CACHE_DISALLOWED_ENTRIES: [&'static str; 6] = [
    "private",
    "no-store",
    "no-cache",
    "max-age=0",
    "must-revalidate",
    "proxy-revalidate",
];
const MAX_AGE_ENTRY: &'static str = "max-age=";

// Appends the header key value pair to a header_lines that ends with the \r\n
pub fn append_header(header_lines: String, key: &String, value: &String) -> String {
    let stripped = header_lines[..header_lines.len() - HttpParser::CRLF_LEN].to_owned();
    format!(
        "{}{}: {}{}{}",
        stripped,
        key,
        value,
        HttpParser::CRLF,
        HttpParser::CRLF
    )
}

pub struct CacheControlHeader {
    words: Vec<String>,
}

impl CacheControlHeader {
    // Task 3: Cache-control parser helper functions
    // Special parse for cache header: Split by comma, and treat quoted string
    // as 1 token
    // Rules from RFC9110:
    // Without quotation mark: "!" / "#" / "$" / "%" / "&" / "'" / "*"
    //  / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
    //  / DIGIT / ALPHA
    // With quotation mark: Any character, except " and \
    // If there is backlash, ignore all rules and treat next char as character
    // Should only see backlash inside quotation mark
    fn cache_control_split(cache_header: &String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut result = vec![];
        let mut current = "".to_string();

        let mut ptr = 0;
        let mut is_quoted = false;
        let text = cache_header.as_bytes().to_vec();
        while ptr < text.len() {
            // Guaranteed to be safe with loop condition
            let c = char::from(text[ptr]); 
            let c_str = c.to_string();
            match (is_quoted, c) {
                (true, '"') => {
                    is_quoted = false;
                    current += &c_str;
                }
                (true, '\\') => {
                    ptr += 1;
                    current += &text.get(ptr)
                        .ok_or("Backlash at the end of line")?.to_string();
                }
                (false, '"') => {
                    is_quoted = true;
                    current += &c_str;
                }
                (false, ',') => {
                    result.push(current.trim().to_lowercase().to_string());
                    current = "".to_string();
                }
                _ => {
                    current += &c_str;
                }
            }

            ptr += 1;
        }

        if !current.trim().is_empty() {
            result.push(current.trim().to_lowercase().to_string());
        }

        Ok(result)
    }

    // Create an entry for the Cache-Control header
    pub fn new(cache_header: &String) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            words: Self::cache_control_split(cache_header)?,
        })
    }
    
    // Whether this response should be cached given the header
    pub fn should_cache(self: &CacheControlHeader) -> bool {
        for word in &self.words {
            for disallowed_entry in CACHE_DISALLOWED_ENTRIES {
                if word.contains(disallowed_entry){
                    return false;
                }
            }
        }
        
        true
    }
    
    // Returns the value for the cache_expire entry
    pub fn cache_expire(self: &CacheControlHeader) -> Option<u32> {
        for cache_directive in &self.words {
            if !cache_directive.contains(MAX_AGE_ENTRY) {
                continue;
            }

            let prefix_len = MAX_AGE_ENTRY.len();
            // Ignore invalid max-age
            return cache_directive[prefix_len..].parse::<u32>().ok();
        }

        None
    }
}
