// Uncomment this block to pass the first stage
use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::SystemTime};

struct Redis {
    data: HashMap<String, (String, Option<SystemTime>)>,
}

impl Redis {
    fn new() -> Redis {
        Redis {
            data: HashMap::new(),
        }
    }

    fn handle_client(&mut self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 { break; }
                    let rec = String::from_utf8_lossy(&buffer[..n]);
                    let res = self.handle_input(rec.to_string());
                    stream.write(res.as_bytes()).unwrap();
                }
                Err(e) => {
                    println!("error: {}", e);
                    return;
                }
            }
        }
    }

    fn handle_input(&mut self, input: String) -> String {
        let words = get_words(input);
        match words[0].as_str() {
            "ECHO" => self.handle_echo(&words),
            "SET" => self.handle_set(&words),
            "GET" => self.handle_get(&words),
            "PING" => make_simple_string("PONG"),
            _ => make_null_bulk_string(),
        }
    }

    fn handle_echo(&self, words: &[String]) -> String {
        make_bulk_string(words[words.len() - 1].as_str())
    }

    fn handle_set(&mut self, words: &[String]) -> String {
        if words.len() == 3 {
            self.run_set(words[1].as_str(), words[2].as_str(), None)
        } else if words.len() == 5 {
            self.run_set(words[1].as_str(), words[2].as_str(), Some(words[4].as_str()))
        } else {
            make_null_bulk_string()
        }
    }

    fn handle_get(&mut self, words: &[String]) -> String {
        if words.len() == 2 {
            self.run_get(&words[1])
        } else {
            make_null_bulk_string()
        }
    }

    fn run_set(&mut self, key: &str, value: &str, expire: Option<&str>) -> String {
        let expiry_time = match expire {
            Some(exp) => match exp.parse::<u64>() {
                Ok(ms) => SystemTime::now().checked_add(std::time::Duration::from_millis(ms)),
                Err(_) => return make_simple_string("ERROR: invalid expire time")
            },
            None => None,
        };
        
        self.data.insert(key.to_string(), (value.to_string(), expiry_time));
        make_simple_string("OK")
    }

    fn run_get(&mut self, key: &str) -> String {
        if let Some((value, Some(expire))) = self.data.get(key) {
            if *expire <= SystemTime::now() {
                self.data.remove(key);
                return make_null_bulk_string();
            }
            make_bulk_string(value)
        } else if let Some((value, None)) = self.data.get(key) {
            make_bulk_string(value)
        } else {
            make_null_bulk_string()
        }
    }
}

// utility functions

fn make_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

fn make_simple_string(s: &str) -> String {
    format!("+{}\r\n", s)
}

fn make_null_bulk_string() -> String {
    "$-1\r\n".to_string()
}

fn get_words(s: String) -> Vec<String> {
    s.split("\r\n")
        .filter_map(|part| {
            if part.starts_with("$") || part.starts_with("*") || part.is_empty() {
                None
            } else {
                Some(part.to_string())
            }
        })
        .collect()
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    let mut redis = Redis::new();
                    redis.handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}