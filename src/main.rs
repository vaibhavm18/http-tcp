use std::{
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
                let request = String::from_utf8_lossy(&buf[0..bytes_read]);
                println!("Bytes: {} read from addr: {}", bytes_read, client_addr);
                println!("Request: \n{}", request);

                let response = "HTTP/1.1 200 OK\r\n\
                               Content-Type: text/plain\r\n\
                               Content-Length: 42\r\n\
                               Connection: close\r\n\
                               \r\n\
                               Hello God! from Rust server!";

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
