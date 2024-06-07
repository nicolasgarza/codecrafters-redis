use crate::servers::slave::Slave;
use crate::{get_words, make_bulk_string, make_null_bulk_string, make_simple_string};
use base64::prelude::*;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    time::SystemTime,
};

#[allow(dead_code)]
pub struct Redis {
    data: HashMap<String, (String, Option<SystemTime>)>,
    my_port: String,
    slaves: Vec<Slave>,
    master_replid: Option<String>,
    master_repl_offset: Option<u64>,
}

impl Redis {
    pub fn new(
        my_port: String,
        master_replid: Option<String>,
        master_repl_offset: Option<u64>,
    ) -> Redis {
        Redis {
            data: HashMap::new(),
            my_port,
            slaves: Vec::new(),
            master_replid,
            master_repl_offset,
        }
    }

    pub fn handle_client(&mut self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    let rec = String::from_utf8_lossy(&buffer[..n]);
                    self.handle_input(rec.to_string(), &mut stream);
                }
                Err(e) => {
                    println!("error: {}", e);
                    return;
                }
            }
        }
    }

    fn handle_input(&mut self, input: String, stream: &mut TcpStream) {
        let words = get_words(input);
        match words[0].as_str() {
            "ECHO" => self.handle_echo(stream, &words),
            "SET" => self.handle_set(stream, &words),
            "GET" => self.handle_get(stream, &words),
            "INFO" => self.handle_info(stream),
            "REPLCONF" => stream
                .write_all(make_simple_string("OK").as_bytes())
                .unwrap(),
            "PSYNC" => self.handle_psync(stream),
            "PING" => stream
                .write_all(make_simple_string("PONG").as_bytes())
                .unwrap(),
            _ => stream
                .write_all(make_null_bulk_string().as_bytes())
                .unwrap(),
        }
    }

    fn handle_echo(&self, stream: &mut TcpStream, words: &[String]) {
        stream
            .write_all(make_bulk_string(vec![words[words.len() - 1].as_str()]).as_bytes())
            .unwrap();
    }

    fn handle_set(&mut self, stream: &mut TcpStream, words: &[String]) {
        if words.len() == 3 {
            self.run_set(stream, words[1].as_str(), words[2].as_str(), None)
        } else if words.len() == 5 {
            self.run_set(
                stream,
                words[1].as_str(),
                words[2].as_str(),
                Some(words[4].as_str()),
            )
        } else {
            stream
                .write_all(make_null_bulk_string().as_bytes())
                .unwrap();
        }
    }

    fn handle_get(&mut self, stream: &mut TcpStream, words: &[String]) {
        if words.len() == 2 {
            self.run_get(stream, &words[1])
        } else {
            stream
                .write_all(make_null_bulk_string().as_bytes())
                .unwrap();
        }
    }

    fn handle_info(&self, stream: &mut TcpStream) {
        let role = "role:master";
        let mut info: Vec<String> = vec![role.to_string()];

        let master_replid = format!("master_replid:{}", self.master_replid.as_ref().unwrap());
        let master_repl_offset = format!("master_repl_offset:{}", self.master_repl_offset.unwrap());
        info.push(master_replid);
        info.push(master_repl_offset);
        let info_refs: Vec<&str> = info.iter().map(|s| s.as_str()).collect();
        let res = make_bulk_string(info_refs);

        stream.write_all(res.as_bytes()).unwrap();
    }

    fn run_set(&mut self, stream: &mut TcpStream, key: &str, value: &str, expire: Option<&str>) {
        let expiry_time = match expire {
            Some(exp) => match exp.parse::<u64>() {
                Ok(ms) => SystemTime::now().checked_add(std::time::Duration::from_millis(ms)),
                Err(_) => None,
            },
            None => None,
        };

        self.data
            .insert(key.to_string(), (value.to_string(), expiry_time));
        stream
            .write_all(make_simple_string("OK").as_bytes())
            .unwrap();
    }

    fn run_get(&mut self, stream: &mut TcpStream, key: &str) {
        if let Some((value, Some(expire))) = self.data.get(key) {
            if *expire <= SystemTime::now() {
                self.data.remove(key);
                stream
                    .write_all(make_null_bulk_string().as_bytes())
                    .unwrap();
                return;
            }
            stream
                .write_all(make_bulk_string(vec![value]).as_bytes())
                .unwrap();
        } else if let Some((value, None)) = self.data.get(key) {
            stream
                .write_all(make_bulk_string(vec![value]).as_bytes())
                .unwrap();
        } else {
            stream
                .write_all(make_null_bulk_string().as_bytes())
                .unwrap();
        }
    }

    fn handle_psync(&mut self, stream: &mut TcpStream) {
        let res = make_simple_string(
            format!("FULLRESYNC {} 0", self.master_replid.as_ref().unwrap()).as_str(),
        );
        stream.write_all(res.as_bytes()).unwrap();
        self.send_rdb(stream);
    }

    fn send_rdb(&self, stream: &mut TcpStream) {
        let base64_rdb = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";
        let content = BASE64_STANDARD.decode(base64_rdb).unwrap();
        let response = format!("${}\r\n", content.len());
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(&content).unwrap();
        println!("wrote rdb");
    }

    // TODO: func to add slave to struct
}
