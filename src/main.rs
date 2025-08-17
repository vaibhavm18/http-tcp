use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process, thread,
    time::Duration,
};

use http_request::HttpRequest;

mod http_request;

fn main() {
    let addr = "127.0.0.1:8080";

    let listener = match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("Listening for connections on {}\n", addr);
            listener
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            process::exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection lost: {}", e);
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

    // Set read timeout to prevent hanging
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(30))) {
        println!("Failed to set read timeout: {}", e);
        return;
    }

    match parse_http_request_robust(&mut stream) {
        Ok(_request) => {
            println!("Request from {}:\n ", client_addr);
            let response = "HTTP/1.1 200 OK\r\n\
                           Content-Type: text/plain\r\n\
                           Content-Length: 29\r\n\
                           Connection: close\r\n\
                           \r\n\
                           Hello God! from Rust server!\n";

            if let Err(e) = stream.write_all(response.as_bytes()) {
                println!("Failed to send response: {}", e);
            }
        }
        Err(e) => {
            println!("Error parsing request from {}: {}", client_addr, e);

            // Send 400 Bad Request
            let error_response = "HTTP/1.1 400 Bad Request\r\n\
                                 Content-Type: text/plain\r\n\
                                 Content-Length: 11\r\n\
                                 Connection: close\r\n\
                                 \r\n\
                                 Bad Request";
            let _ = stream.write_all(error_response.as_bytes());
        }
    }
    println!("-----");
}

fn parse_http_request_robust(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut reader = BufReader::new(stream);

    // Read request line
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| format!("Failed to read request line: {}", e))?;

    if request_line.is_empty() {
        return Err("Empty request line".to_string());
    }

    // Remove trailing \r\n
    request_line = request_line.trim_end().to_string();

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid request line format".to_string());
    }

    let (method, path, version) = (
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    );

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut header_line = String::new();
        reader
            .read_line(&mut header_line)
            .map_err(|e| format!("Failed to read header: {}", e))?;

        header_line = header_line.trim_end().to_string();

        // Empty line indicates end of headers
        if header_line.is_empty() {
            break;
        }

        if let Some((key, value)) = header_line.split_once(": ") {
            headers.insert(key.to_lowercase(), value.to_string());
        } else {
            return Err(format!("Invalid header format: {}", header_line));
        }
    }

    // Read body based on Content-Length or Transfer-Encoding
    let body = read_http_body(&mut reader, &headers)?;

    let request = HttpRequest::builder()
        .method(method)
        .path(path)
        .version(version)
        .body(body)
        .headers(headers)
        .build();

    match request {
        Ok(request) => Ok(request),
        Err(e) => Err(e.to_string()),
    }
}

fn read_http_body(
    reader: &mut BufReader<&mut TcpStream>,
    headers: &HashMap<String, String>,
) -> Result<Option<Vec<u8>>, String> {
    // Check for Transfer-Encoding: chunked
    if let Some(transfer_encoding) = headers.get("transfer-encoding") {
        if transfer_encoding.to_lowercase() == "chunked" {
            return read_chunked_body(reader);
        }
    }

    // Check for Content-Length
    if let Some(content_length_str) = headers.get("content-length") {
        let content_length: usize = content_length_str
            .parse()
            .map_err(|_| "Invalid Content-Length header".to_string())?;

        if content_length == 0 {
            return Ok(None);
        }

        return read_fixed_length_body(reader, content_length);
    }

    // No body indicators found
    Ok(None)
}

fn read_fixed_length_body(
    reader: &mut BufReader<&mut TcpStream>,
    content_length: usize,
) -> Result<Option<Vec<u8>>, String> {
    let mut body = vec![0u8; content_length];
    reader
        .read_exact(&mut body)
        .map_err(|e| format!("Failed to read body: {}", e))?;

    Ok(Some(body))
}

fn read_chunked_body(reader: &mut BufReader<&mut TcpStream>) -> Result<Option<Vec<u8>>, String> {
    let mut body = Vec::new();

    loop {
        // Read chunk size line
        let mut chunk_size_line = String::new();
        reader
            .read_line(&mut chunk_size_line)
            .map_err(|e| format!("Failed to read chunk size: {}", e))?;

        let chunk_size_line = chunk_size_line.trim();

        // Parse chunk size (hex)
        let chunk_size = if let Some(semicolon_pos) = chunk_size_line.find(';') {
            // Remove chunk extensions (everything after ';')
            &chunk_size_line[..semicolon_pos]
        } else {
            chunk_size_line
        };

        let chunk_size = usize::from_str_radix(chunk_size, 16)
            .map_err(|_| format!("Invalid chunk size: {}", chunk_size))?;

        // If chunk size is 0, we've reached the end
        if chunk_size == 0 {
            // Read trailing headers (if any) until empty line
            loop {
                let mut trailer_line = String::new();
                reader
                    .read_line(&mut trailer_line)
                    .map_err(|e| format!("Failed to read trailer: {}", e))?;

                if trailer_line.trim().is_empty() {
                    break;
                }
            }
            break;
        }

        // Read chunk data
        let mut chunk_data = vec![0u8; chunk_size];
        reader
            .read_exact(&mut chunk_data)
            .map_err(|e| format!("Failed to read chunk data: {}", e))?;

        body.extend_from_slice(&chunk_data);

        // Read trailing CRLF after chunk data
        let mut crlf = String::new();
        reader
            .read_line(&mut crlf)
            .map_err(|e| format!("Failed to read chunk CRLF: {}", e))?;
    }

    if body.is_empty() {
        Ok(None)
    } else {
        Ok(Some(body))
    }
}
