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
            Ok(_msg) => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
                return;
            }
        }
    }
}