// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                let rec = String::from_utf8_lossy(&buffer[..n]);
                println!("received: {}", rec);
                let words = get_words(rec.to_string());
                match words[0].as_str() {
                    "ECHO" => {
                        let res = make_bulk_string(words[words.len() - 1].as_str());
                        let _ = stream.write_all(res.as_bytes());
                    },
                    "PING" => {
                        let res = make_simple_string("PONG");
                        stream.write_all(res.as_bytes()).unwrap();
                    },
                    _ => {
                        let res = make_simple_string("ERROR");
                        stream.write_all(res.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
                return;
            }
        }
    }
}

fn make_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

fn make_simple_string(s: &str) -> String {
    format!("+{}\r\n", s)
}

fn get_words(s: String) -> Vec<String> {
    let parts: Vec<&str> = s.split("\r\n").collect();

    let mut iter = parts.iter();
    let mut words = Vec::new();
    while let Some(part) = iter.next() {
        if part.starts_with("$") {
            if let Some(word) = iter.next() {
                words.push(word.to_string());
            }
        }
    }

    words
}