// Uncomment this block to pass the first stage
use std::{collections::HashMap, env, io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::SystemTime};

struct Redis {
    data: HashMap<String, (String, Option<SystemTime>)>,
    is_master: bool,
    master_address: Option<String>,
    slaves: Vec<TcpStream>,
}

impl Redis {
    fn new(is_master: bool, master_address: Option<String>) -> Redis {
        Redis {
            data: HashMap::new(),
            is_master,
            master_address,
            slaves: Vec::new(),
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
                    stream.write_all(res.as_bytes()).unwrap();
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
            "INFO" => self.handle_info(),
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

    fn handle_info(&self) -> String {
        let role = if self.is_master {
            "master"
        } else {
            "slave"
        };
        let info = format!("role:{}", role);
        make_bulk_string(&info)
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
    let args: Vec<String> = env::args().collect();
    let mut port = String::from("6379");
    let mut is_master = true;
    let mut master_address: Option<String> = None;
    println!("{:?}", args);
    if args.len() > 1 {
        for i in 0..args.len() {
            if args[i] == "--port" && i + 1 < args.len() {
                port = args[i + 1].clone();
            } else if args[i] == "--replicaof" && i + 1 < args.len() {
                is_master = false;
                let parts: Vec<&str> = args[i + 1].split_whitespace().collect();
                let port = parts[1].trim_end_matches('"').to_string();
                master_address = Some(port);
            }
        }
    }

    let address = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&address).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let is_master = is_master;
                let master_address = master_address.clone();
                thread::spawn(move || {
                    let mut redis = Redis::new(
                        is_master,
                        master_address,
                    );
                    redis.handle_client(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}