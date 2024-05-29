// Uncomment this block to pass the first stage
use std::{collections::HashMap, env, io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::SystemTime};
use crate::servers::{slave::Slave, redis::Redis};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut port = String::from("6379");
    let mut is_master = true;
    let mut master_address: Option<String> = None;
    let mut master_replid: Option<String> = None;
    let mut master_repl_offset: Option<u64> = None;

    if args.len() > 1 {
        for i in 0..args.len() {
            if args[i] == "--port" && i + 1 < args.len() {
                port = args[i + 1].clone();
            } else if args[i] == "--replicaof" && i + 1 < args.len() {
                is_master = false;
                let parts: Vec<&str> = args[i + 1].split_whitespace().collect();
                let port = format!("127.0.0.1:{}", parts[1].trim_end_matches('"').to_string());
                master_address = Some(port);
            }
        }
    }

    if is_master {
        master_replid = Some("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string());
        master_repl_offset = Some(0);
    } 

    let address = format!("127.0.0.1:{}", port);

    let listener = TcpListener::bind(&address).unwrap();

    if !is_master {
        if let Some(_master_addr) = &master_address {
            let init_redis = Redis::new(is_master, master_address.clone(), port.clone(), master_replid.clone(), master_repl_offset.clone());
            init_redis.ping_master();
        }
    }

    for stream in listener.incoming() {

        match stream {
            Ok(stream) => {
                let local_is_master = is_master;
                let local_master_address = master_address.clone();
                let local_port = port.clone();
                let local_master_replid = master_replid.clone();
                let local_master_repl_offset = master_repl_offset.clone();
                thread::spawn(move || {
                    let mut thread_redis = Redis::new(local_is_master, local_master_address, local_port, local_master_replid, local_master_repl_offset);
                    thread_redis.handle_client(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}