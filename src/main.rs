// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    //
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.write_all(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

// fn handle_client(mut stream: TcpStream) {
//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();
//     let rec = String::from_utf8_lossy(&buffer);
//     if rec.starts_with("PING") {
//         stream.write_all(b"+PONG\r\n").unwrap();
//     } else {
//         stream.write_all(b"-ERR unknown command\r\n").unwrap();
//     }
// }