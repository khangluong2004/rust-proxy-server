mod cache;
mod http_parser;
mod lru_queue;
mod proxy;
mod request;
mod response;
mod headers;

use crate::proxy::Proxy;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let mut port = 0u16;
    let mut does_cache = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => {
                if i + 1 >= args.len() {
                    return Err("-p need a port".into());
                }
                port = args[i + 1].parse::<u16>()?;
                i += 2;
            }
            "-c" => {
                does_cache = true;
                i += 1;
            }
            _ => {
                return Err(format!("unknown argument {}", args[i]).into());
            }
        }
    }

    // # 772, no global panic catch then
    let mut proxy = Proxy::new(does_cache);
    proxy.start_server(port)
}
