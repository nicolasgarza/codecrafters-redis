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
            "ECHO" => {
                make_bulk_string(words[words.len() - 1].as_str())
            },
            "SET" => {
                if words.len() == 3 {
                    self.run_set(words[1].as_str(), words[2].as_str(), None)
                } else if words.len() == 5 {
                    self.run_set(words[1].as_str(), words[2].as_str(), Some(words[4].as_str()))
                } else {
                    make_simple_string("-1")
                }
            },
            "GET" => {
                self.run_get(words[1].as_str())
            }
            "PING" => {
                make_simple_string("PONG")
            },
            _ => {
                make_simple_string("-1")
            }
        }
    }

    fn run_set(&mut self, key: &str, value: &str, expire: Option<&str>) -> String {
        match expire {
            Some(expire) => {
                let expire = expire.parse::<u64>().unwrap();
                let expire = SystemTime::now().checked_add(std::time::Duration::from_secs(expire)).unwrap();
                self.data.insert(key.to_string(), (value.to_string(), Some(expire)));
                make_simple_string("OK")
            },
            None => {
                self.data.insert(key.to_string(), (value.to_string(), None));
                make_simple_string("OK")
            }
        }
    }

    fn run_get(&mut self, key: &str) -> String {
        match self.data.get(key) {
            Some(value) => {
                println!("{:?}", value.1);
                match value.1 {
                    Some(expire) => {
                        if expire > SystemTime::now() {
                            make_bulk_string(value.0.as_str())
                        } else {
                            self.data.remove(key);
                            make_simple_string("-1")
                        }
                    },
                    None => {
                        make_bulk_string(value.0.as_str())
                    }
                }
            },
            None => {
                make_simple_string("ERROR")
            }
        }
    }
    
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
