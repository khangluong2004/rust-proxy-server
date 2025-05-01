mod request;
mod response;
mod http_parser;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::string::ParseError;
use std::time::Duration;
use crate::http_parser::HttpParser;

struct CliArguments {
    port: u16,
    does_cache: bool,
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
