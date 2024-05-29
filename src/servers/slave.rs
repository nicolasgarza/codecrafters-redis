use std::{io::{Read, Write}, net::TcpStream};

use crate::{make_bulk_string, make_simple_string, make_null_bulk_string, make_resp_array, get_words};

pub struct Slave {
    stream: TcpStream,
    address: String,
    master_address: String,
    my_port: String,
    replication_id: String,
    replication_offset: u64,
}

impl Slave {
    fn new(address: String, master_address: String, my_port: String) -> Slave {
        let stream = TcpStream::connect(address.as_ref()).unwrap();
        Slave {
            stream,
            address,
            master_address,
            my_port,
            replication_id: String::new(),
            replication_offset: 0,
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
            "INFO" => self.handle_info(),
            "PING" => make_simple_string("PONG"),
            "FULLRESYNC" => self.handle_full_resync(&words),
            _ => make_null_bulk_string(),
        }
    }

    fn handle_echo(&self, words: &[String]) -> String {
        make_bulk_string(vec![words[words.len() - 1].as_str()])
    }

    fn handle_info(&self) -> String {
        let role = "role:slave";
        let info: Vec<String> = vec![role.to_string()];

        let info_refs: Vec<&str> = info.iter().map(|s| s.as_str()).collect();
        let res = make_bulk_string(info_refs);
    
        res
    }

    fn ping_master(&self) {
        let mut stream = TcpStream::connect(self.master_address.as_ref().unwrap()).unwrap();
        let ping = "*1\r\n$4\r\nPING\r\n";
        stream.write_all(ping.as_bytes()).unwrap();
        
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(n) => {
                let rec = String::from_utf8_lossy(&buffer[..n]);
                if rec != "+PONG\r\n" {
                    println!("Error: Master did not respond with PONG");
                }
                self.send_conf_to_master(stream);
            }
            Err(e) => {
                println!("error: {}", e);
                return;
            }
        }
    }

    fn send_conf_to_master(&self, mut stream: TcpStream) {
        // Send the first REPLCONF command
        let res1 = make_resp_array(vec!["REPLCONF", "listening-port", &self.my_port]);
        stream.write_all(res1.as_bytes()).unwrap();
        // Wait and read the response
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        let response1 = String::from_utf8_lossy(&buffer[..n]);
        if response1 != "+OK\r\n" {
            println!("Error: Master did not respond with OK after REPLCONF listening-port");
        }

        // Send the second REPLCONF command
        let res2 = make_resp_array(vec!["REPLCONF", "capa", "psync2"]);
        stream.write_all(res2.as_bytes()).unwrap();
        // Wait and read the response
        let n = stream.read(&mut buffer).unwrap();
        let response2 = String::from_utf8_lossy(&buffer[..n]);
        if response2 != "+OK\r\n" {
            println!("Error: Master did not respond with OK after REPLCONF capa psync2");
        }

        self.send_psync_to_master(stream);
    }

    fn send_psync_to_master(&self, mut stream: TcpStream) {
        let res = make_resp_array(vec!["PSYNC", "?", "-1"]);
        stream.write_all(res.as_bytes()).unwrap();

        let mut buffer = [0; 1024];
        let n  = stream.read(&mut buffer).unwrap();
        let _ = String::from_utf8_lossy(&buffer[..n]);

    }

    fn handle_full_resync(&mut self, words: &[String]) -> String {
        self.replication_id = words[1].clone();
        self.replication_offset = words[2].parse().unwrap();
        make_simple_string("OK")
    }
}