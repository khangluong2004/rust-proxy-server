mod request;
mod response;
mod http_parser;
mod proxy;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use crate::proxy::Proxy;

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
    
    let mut proxy = Proxy::new(does_cache);
    proxy.start_server(port)
}
