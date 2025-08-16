use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process, thread,
};

fn main() {
    let addr = "127.0.0.1:8080";

    let listner = match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("Listening for connections on  {}\n", addr);
            listener
        }
        Err(e) => {
            println!("Connection failed.{}", e);
            process::exit(1);
        }
    };

    for stream in listner.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection lost.{}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let client_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(e) => {
            println!("Error while getting client addr: {}", e);
            return;
        }
    };

    let mut buf = [0; 1024];

    match stream.read(&mut buf) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                let request = String::from_utf8_lossy(&buf[0..bytes_read]).into_owned();
                println!("Bytes: {} read from addr: {}", bytes_read, client_addr);
                let request = match parse_http_request(request) {
                    Ok(request) => request,
                    Err(e) => {
                        println!("Error: {}", e);
                        return;
                    }
                };

                println!("{:?}", request);
                let response = "HTTP/1.1 200 OK\r\n\
                               Content-Type: text/plain\r\n\
                               Content-Length: 29\r\n\
                               Connection: close\r\n\
                               \r\n\
                               Hello God! from Rust server!\n";

                if let Err(e) = stream.write_all(response.as_bytes()) {
                    println!("Failed to send reponse: {}", e);
                }
                println!("-----");
            }
        }
        Err(e) => {
            println!("Error whilte reading bytes: {}", e);
            println!("-----");
        }
    }
}

#[derive(Debug, PartialEq)]
struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

fn parse_http_request(request: String) -> Result<HttpRequest, String> {
    let requests: Vec<&str> = request.split("\r\n").collect();
    if requests.is_empty() {
        return Err("Request is empty.".to_string());
    }
    let request_line = requests[0];
    let parts: Vec<&str> = request_line.split(" ").collect();

    if parts.len() != 3 {
        return Err("Invalid request line format".to_string());
    }

    let (method, path, version) = (
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    );

    let mut headers = HashMap::new();
    let mut header_end = 1;

    for (i, line) in requests.iter().enumerate().skip(1) {
        if line.is_empty() {
            header_end = i;
            break;
        }

        if let Some((key, val)) = line.split_once(": ") {
            headers.insert(key.to_lowercase(), val.to_string());
        }
    }

    let body = if header_end + 1 < requests.len() {
        let body_lines = &requests[header_end + 1..];
        if !body_lines.is_empty() && !body_lines.join("\n").trim().is_empty() {
            Some(body_lines.join("\n"))
        } else {
            None
        }
    } else {
        None
    };

    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}
