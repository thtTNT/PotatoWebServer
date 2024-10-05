use crate::config::Config;
use crate::network;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::net::TcpStream;

const HTTP_VERSION: &str = "HTTP/1.1";

pub trait Proxy {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    fn get_head(&self) -> Result<ResponseHead, Error>;
}

pub struct FileProxy {
    path: String,
    file: File,
    http_status: HttpStatus,
}

impl Proxy for FileProxy {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }

    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        panic!("Cannot write to file proxy");
    }

    fn get_head(&self) -> Result<ResponseHead, Error> {
        let mut headers = vec![];
        let content_length = self.file.metadata()?.len();
        headers.push(("Content-Length".to_string(), content_length.to_string()));
        headers.push(("Content-Type".to_string(), MineType::from_path(&self.path).to_string()));
        Ok(ResponseHead {
            version: HTTP_VERSION.to_string(),
            status: self.http_status,
            headers,
        })
    }
}

impl FileProxy {
    pub fn new(path: &String) -> Box<dyn Proxy> {
        let file = File::open(&path);
        match file {
            Ok(file) => Box::new(FileProxy {
                path: path.clone(),
                file,
                http_status: HttpStatus::Ok,
            }),
            Err(_) => {
                let path_404 = String::from(&Config::global().root_dir) + &Config::global().error_pages["404"];
                let file = File::open(&path_404);
                match file {
                    Ok(file) => Box::new(FileProxy {
                        path: path_404.clone(),
                        file,
                        http_status: HttpStatus::NotFound,
                    }),
                    Err(_) => TextProxy::new(&String::from("404 Not Found"), HttpStatus::NotFound),
                }
            }
        }
    }
}

pub struct TextProxy {
    text: String,
    pointer: usize,
    http_status: HttpStatus,
}

impl Proxy for TextProxy {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = self.text.as_bytes();
        let bytes_len = bytes.len();
        let buf_len = buf.len();
        let remaining = bytes_len - self.pointer;
        let to_copy = std::cmp::min(remaining, buf_len);
        buf[..to_copy].copy_from_slice(&bytes[self.pointer..self.pointer + to_copy]);
        self.pointer += to_copy;
        Ok(to_copy)
    }

    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        panic!("Cannot write to text proxy");
    }

    fn get_head(&self) -> Result<ResponseHead, Error> {
        let mut headers = vec![];
        headers.push(("Content-Length".to_string(), self.text.len().to_string()));
        headers.push(("Content-Type".to_string(), "text/plain".to_string()));
        Ok(ResponseHead {
            version: HTTP_VERSION.to_string(),
            status: self.http_status,
            headers,
        })
    }
}

impl TextProxy {
    pub fn new(text: &String, status: HttpStatus) -> Box<dyn Proxy> {
        Box::new(TextProxy {
            text: text.clone(),
            pointer: 0,
            http_status: status,
        })
    }
}


#[derive(Debug, Copy, Clone)]
pub enum HttpStatus {
    Ok,
    Created,
    Accepted,
    NoContent,
    MovedPermanently,
    Found,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
}

impl HttpStatus {
    pub fn string(&self) -> String {
        match self {
            HttpStatus::Ok => "OK".to_string(),
            HttpStatus::Created => "Created".to_string(),
            HttpStatus::Accepted => "Accepted".to_string(),
            HttpStatus::NoContent => "No Content".to_string(),
            HttpStatus::MovedPermanently => "Moved Permanently".to_string(),
            HttpStatus::Found => "Found".to_string(),
            HttpStatus::NotModified => "Not Modified".to_string(),
            HttpStatus::BadRequest => "Bad Request".to_string(),
            HttpStatus::Unauthorized => "Unauthorized".to_string(),
            HttpStatus::Forbidden => "Forbidden".to_string(),
            HttpStatus::NotFound => "Not Found".to_string(),
            HttpStatus::MethodNotAllowed => "Method Not Allowed".to_string(),
            HttpStatus::InternalServerError => "Internal Server Error".to_string(),
            HttpStatus::NotImplemented => "Not Implemented".to_string(),
            HttpStatus::BadGateway => "Bad Gateway".to_string(),
            HttpStatus::ServiceUnavailable => "Service Unavailable".to_string(),
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            HttpStatus::Ok => 200,
            HttpStatus::Created => 201,
            HttpStatus::Accepted => 202,
            HttpStatus::NoContent => 204,
            HttpStatus::MovedPermanently => 301,
            HttpStatus::Found => 302,
            HttpStatus::NotModified => 304,
            HttpStatus::BadRequest => 400,
            HttpStatus::Unauthorized => 401,
            HttpStatus::Forbidden => 403,
            HttpStatus::NotFound => 404,
            HttpStatus::MethodNotAllowed => 405,
            HttpStatus::InternalServerError => 500,
            HttpStatus::NotImplemented => 501,
            HttpStatus::BadGateway => 502,
            HttpStatus::ServiceUnavailable => 503,
        }
    }
}


pub(crate) enum MineType {
    Html,
    Css,
    Js,
    Jpg,
    Png,
    Gif,
    Svg,
    Other,
}

impl MineType {
    pub fn from_path(path: &str) -> MineType {
        if path.ends_with(".html") {
            MineType::Html
        } else if path.ends_with(".css") {
            MineType::Css
        } else if path.ends_with(".js") {
            MineType::Js
        } else if path.ends_with(".jpg") {
            MineType::Jpg
        } else if path.ends_with(".png") {
            MineType::Png
        } else if path.ends_with(".gif") {
            MineType::Gif
        } else if path.ends_with(".svg") {
            MineType::Svg
        } else {
            MineType::Other
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            MineType::Html => "text/html".to_string(),
            MineType::Css => "text/css".to_string(),
            MineType::Js => "text/javascript".to_string(),
            MineType::Jpg => "image/jpeg".to_string(),
            MineType::Png => "image/png".to_string(),
            MineType::Gif => "image/gif".to_string(),
            MineType::Svg => "image/svg+xml".to_string(),
            MineType::Other => "application/octet-stream".to_string(),
        }
    }
}

pub struct ResponseHead {
    pub version: String,
    pub status: network::HttpStatus,
    pub headers: Vec<(String, String)>,
}


pub fn response(tcp_stream: &mut TcpStream, proxy: &mut dyn Proxy) -> ResponseHead {
    let head = proxy.get_head();
    if head.is_err() {
        panic!("Failed to get head: {}", head.err().unwrap());
    }
    let head = head.unwrap();

    let status_line = format!("{} {} {}\r\n", head.version, head.status.code(), head.status.string());
    let headers = head.headers.iter().map(|(k, v)| format!("{}: {}\r\n", k, v)).collect::<String>();
    let headers = format!("{}\r\n", headers);

    tcp_stream.write_all(status_line.as_bytes()).unwrap();
    tcp_stream.write_all(headers.as_bytes()).unwrap();

    let mut buf = [0; 1024];
    loop {
        let bytes_read = proxy.read(&mut buf).unwrap();
        if bytes_read == 0 {
            break;
        }
        tcp_stream.write_all(&buf[..bytes_read]).unwrap();
    }
    tcp_stream.flush().unwrap();
    head
}
