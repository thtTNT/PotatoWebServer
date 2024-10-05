mod network;
mod config;

use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader};
use std::thread;
use crate::network::{FileProxy, HttpStatus};

#[derive(Debug)]
enum ReadState {
    RequestLine,
    Headers,
    Done,
}

#[derive(Debug)]
struct Request {
    state: ReadState,
    method: Option<String>,
    path: Option<String>,
    version: Option<String>,
    headers: Vec<(String, String)>,
}


fn handle_request(client: Request, stream: &mut TcpStream) {
    let method = client.method.unwrap();
    let original_path = client.path.unwrap();
    let mut path = original_path.clone();
    let version = client.version.unwrap();

    if path.ends_with("/") {
        path = String::from(path) + &config::Config::global().home_page;
    }

    path = String::from(&config::Config::global().root_dir) + &*path;

    let mut proxy = FileProxy::new(&path);
    let head = network::response(stream, proxy.as_mut());
    println!("{} {} {} {} {}", method, original_path, version, head.status.code(), head.status.string());
}

fn handle_client(mut stream: TcpStream) {
    let mut client = Request {
        state: ReadState::RequestLine,
        method: None,
        path: None,
        version: None,
        headers: vec![],
    };
    let reader = BufReader::new(&stream);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                match client.state {
                    ReadState::RequestLine => {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() != 3 {
                            println!("Invalid request line: {}", line);
                            return;
                        }
                        client.method = Some(parts[0].to_string());
                        client.path = Some(parts[1].to_string());
                        client.version = Some(parts[2].to_string());
                        client.state = ReadState::Headers;
                    }
                    ReadState::Headers => {
                        if line.is_empty() {
                            client.state = ReadState::Done;
                            handle_request(client, &mut stream);
                            return;
                        } else {
                            let op = line.split_once(":");
                            match op {
                                None => {
                                    println!("Invalid header: {}", line);
                                    return;
                                }
                                Some(parts) => {
                                    client.headers.push((parts.0.to_string(), parts.1.to_string()));
                                }
                            }
                        }
                    }
                    ReadState::Done => {
                        println!("Not implemented");
                        return;
                    }
                }
            }
            Err(e) => {
                println!("Failed to read line: {}", e);
                return;
            }
        }
    }
}

fn main() {
    let config = config::Config::global();

    let ip_address = config.host.clone() + ":" + &config.port.to_string();
    let listener_res = TcpListener::bind(&ip_address);
    if listener_res.is_err() {
        println!("Failed to bind to address: {}", listener_res.err().unwrap());
        return;
    }
    let listener = listener_res.unwrap();
    println!("Listening on {ip_address}");


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Failed to establish a connection: {}", e);
            }
        }
    }
}
