use std::{env, net::TcpListener, thread};

use redis_starter_rust::servers::redis::Redis;
use redis_starter_rust::servers::slave::Slave;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut port = String::from("6379");  // Default port
    let mut is_master = true;
    let mut master_address: Option<String> = None;
    let mut master_replid: Option<String> = None;
    let mut master_repl_offset: Option<u64> = None;

    // Argument parsing
    for i in 0..args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            port = args[i + 1].clone();
        } else if args[i] == "--replicaof" && i + 1 < args.len() {
            is_master = false;
            let parts: Vec<&str> = args[i + 1].split_whitespace().collect();
            if parts.len() > 1 {
                master_address = Some(format!("{}:{}", parts[0], parts[1]));
            }
        }
    }

    if is_master {
        master_replid = Some("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string());
        master_repl_offset = Some(0);
    }

    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address).unwrap();

    // Handling based on role
    if is_master {
        println!("Running as Master on {}", address);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let local_port = port.clone();
                    let local_replid = master_replid.clone();
                    thread::spawn(move || {
                        let mut master_redis = Redis::new(local_port, local_replid, master_repl_offset);
                        master_redis.handle_client(stream);
                    });
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    } else {
        println!("Running as Slave on port {}, connecting to Master at {:?}", port, master_address);
        if let Some(master_addr) = master_address.clone() {
            let local_port = port.clone();
            thread::spawn(move || {
                let slave = Slave::new(master_addr, local_port.clone());
                slave.ping_master(); // Initialize and maintain master connection here
            });
        }
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let local_port = port.clone();
                    let local_master_address = master_address.clone();
                    thread::spawn(move || {
                        let mut slave_redis = Slave::new(local_master_address.clone().unwrap(), local_port.clone());
                        slave_redis.handle_client(stream); // Dedicated to handling client commands
                    });
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    }


}