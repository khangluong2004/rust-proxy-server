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
use std::panic;

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

    // Paranoid error check
    loop {
        if let Ok(some) = panic::catch_unwind(||->Result<(), Box<dyn Error>> {
            let mut proxy = Proxy::new(does_cache);
            proxy.start_server(port)?;
            Ok(())
        }) {
            match some {
                Ok(_) => {},
                Err(err) => {print!("{:?}. Caught error! Restart proxy", err)}
            }
        } else {
            println!("Caught panic! Restart proxy");
        };
    }
}
