mod network;
mod config;

use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Read, Write};
use std::thread;

#[derive(Debug)]
enum ReadState {
    Metadata,
    Headers,
    Done,
}

#[derive(Debug)]
struct Client {
    state: ReadState,
    method: Option<String>,
    path: Option<String>,
    version: Option<String>,
    headers: Vec<(String, String)>,
}

fn handle_request(client: Client, stream: &mut TcpStream) {
    let method = client.method.unwrap();
    let mut path = client.path.unwrap();
    let version = client.version.unwrap();

    println!("{} {} {}", method, path, version);

    if path.ends_with("/") {
        path = String::from(path) + &config::Instance::global().home_page;
    }

    path = String::from(&config::Instance::global().root_dir) + &*path;

    let file = std::fs::read(&path).unwrap();
    let content_length = file.len();

    stream.write_all(b"HTTP/1.1 200 OK\r\n").unwrap();
    if path.ends_with(".html") {
        stream.write_all(b"Content-Type: text/html\r\n").unwrap();
    } else if path.ends_with(".css") {
        stream.write_all(b"Content-Type: text/css\r\n").unwrap();
    } else if path.ends_with(".js") {
        stream.write_all(b"Content-Type: text/javascript\r\n").unwrap();
    } else if path.ends_with(".jpg") {
        stream.write_all(b"Content-Type: image/jpeg\r\n").unwrap();
    } else if path.ends_with(".png") {
        stream.write_all(b"Content-Type: image/png\r\n").unwrap();
    } else if path.ends_with(".gif") {
        stream.write_all(b"Content-Type: image/gif\r\n").unwrap();
    } else if path.ends_with(".svg") {
        stream.write_all(b"Content-Type: image/svg+xml\r\n").unwrap();
    } else {
        stream.write_all(b"Content-Type: application/octet-stream\r\n").unwrap();
    }
    stream.write_all(format!("Content-Length: {}\r\n", content_length).as_bytes()).unwrap();
    stream.write_all(b"\r\n").unwrap();
    stream.write_all(&file).unwrap();
    stream.flush().unwrap();
}

fn handle_client(mut stream: TcpStream) {
    let mut client = Client {
        state: ReadState::Metadata,
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
                    ReadState::Metadata => {
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
    let config = config::Instance::global();

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
