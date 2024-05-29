use std::{collections::HashMap, env, io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::SystemTime};
use crate::{make_bulk_string, make_simple_string, make_null_bulk_string, make_resp_array, get_words};
use crate::servers::slave::Slave;

#[allow(dead_code)]
struct Redis {
    data: HashMap<String, (String, Option<SystemTime>)>,
    is_master: bool,
    master_address: Option<String>,
    my_port: String,
    slaves: Vec<Slave>,
    master_replid: Option<String>,
    master_repl_offset: Option<u64>,
}

impl Redis {
    fn new(is_master: bool, master_address: Option<String>, my_port: String, master_replid: Option<String>, master_repl_offset: Option<u64>) -> Redis {
        Redis {
            data: HashMap::new(),
            is_master,
            master_address,
            my_port,
            slaves: Vec::new(),
            master_replid,
            master_repl_offset,
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
            "REPLCONF" => make_simple_string("OK"),
            "PSYNC" => { 
                make_simple_string(format!("FULLRESYNC {} 0", self.master_replid.as_ref().unwrap()).as_str());
                self.send_current_state(/* need to determine how to know which slave to send rdb to */)
            },
            "PING" => make_simple_string("PONG"),
            _ => make_null_bulk_string(),
        }
    }

    fn handle_echo(&self, words: &[String]) -> String {
        make_bulk_string(vec![words[words.len() - 1].as_str()])
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
        let role = "role:master";
        let mut info: Vec<String> = vec![role.to_string()];

        let master_replid = format!("master_replid:{}", self.master_replid.as_ref().unwrap());
        let master_repl_offset = format!("master_repl_offset:{}", self.master_repl_offset.unwrap());
        info.push(master_replid);
        info.push(master_repl_offset);
        let info_refs: Vec<&str> = info.iter().map(|s| s.as_str()).collect();
        let res = make_bulk_string(info_refs);
    
        res
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
            make_bulk_string(vec![value])
        } else if let Some((value, None)) = self.data.get(key) {
            make_bulk_string(vec![value])
        } else {
            make_null_bulk_string()
        }
    }
}
