use rand::{distributions::Alphanumeric, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::net;
use std::env;
use std::time::Instant;

#[derive(PartialEq)]
struct Socket {
    ip: String,
    port: u16
}

struct Session {
    key: String,
    socket: Rc<Socket>,
    password_protected: bool
}

struct Server {
    port: u16,
    sessions: Vec<Session>,
    client_hashes: Vec<String>
}

impl Server {
    //
    // static fn
    //

    fn new(port: u16) -> Server {
        Server { port: port, sessions: Vec::new(), client_hashes: Vec::new() }
    }

    fn generate_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    fn poll(server: &Server) {
        let ipaddr = "0.0.0.0".to_string() + ":" + &server.port.to_string();
        let socket = net::UdpSocket::bind(ipaddr).expect("Failed to bind host socket");
        let mut buf = [0u8; 100];
        let mut last_packet_tm = Instant::now();

        loop {
            let mut read = true;

            while read {
                match socket.recv_from(&mut buf) {
                    Ok((bytes, client_addr)) => {
                        println!("client {} sent bytes {}", client_addr, bytes);
                        println!("content was: {}", std::str::from_utf8(&mut buf[0..bytes]).unwrap());

                        // update our packet time
                        last_packet_tm = Instant::now();
                    },
                    Err(e) => {
                        println!("Error reading bytes: {}", e);
                    }
                }
            }
        }
    }

    //
    // non mut fn
    //

    fn has_key(&self, key: &String) -> bool {
        self.sessions.iter().position(|session: &Session| session.key == *key) != None
    }

    fn has_socket(&self, socket: &Socket) -> bool {
        self.sessions.iter().position(|session: &Session| *session.socket == *socket) != None
    }

    fn valid_client_hash(&self, hash: &String) -> bool {
        self.client_hashes.iter().position(|h: &String| *h == *hash) != None
    }

    //
    // mut fn
    //

    fn support_client_hashes(&mut self, hashes: Vec<String>) {
        self.client_hashes = hashes;
    }

    fn create_session(&mut self, socket: Rc<Socket>, password_protected: bool) -> Option<String> {
        let mut result = None;

        if !self.has_socket(&socket) {
            loop {
                let new_key = Server::generate_key();

                if !self.has_key(&new_key) {
                    let new_session = Session { key: new_key.clone(), socket: socket.clone(), password_protected: password_protected };
                    self.sessions.push(new_session);
                    println!("Session created for client {}:{} with key {} (password_protected: {})", socket.ip, socket.port, new_key, password_protected);
                    result = Some(new_key);
                    break;
                }
            }
        } else {
            println!("Session for {}:{} cannot be created because it already exists", socket.ip, socket.port);
        }

        result
    }
    
    fn get_session(&mut self, key: &String) -> Option<&Session> {
        self.sessions.iter().find(|session: &&Session| session.key == *key)
    }

    fn drop_session(&mut self, socket: &Socket) {
        if let Some(pos) = self.sessions.iter().position(|session: &Session| *session.socket == *socket) {
            self.sessions.remove(pos);
        }
    }
}

fn file_read_lines(path: &str) -> Vec<String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut result = Vec::new();

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors
        result.push(line);
    }

    result
}

fn print_key(key: &Option<String>) {
    match key {
        Some(x) => println!("Key was {}", x),
        None => println!("Empty key")
    }
}

fn test_hash(server: &Server, hash: &String) {
    println!("Hash {} is supported by server: {}", hash, server.valid_client_hash(hash));
}

fn main() {
    let mut port: u16;

    match env::args().nth(1) {
        Some(arg) => {
            match arg.parse::<u16>() {
                Ok(x) => {
                    port = x;
                },
                Err(_) => {
                    println!("Aborting! Port number must be an integer sequence only!");
                    return;
                }
            }
        },
        None => {
            println!("Supply a port number!");
            return;
        }
    }

    let mut server = Server::new(port);

    server.support_client_hashes(file_read_lines("./hashes.txt"));

    // test_hash(&server, &"ABCDEF".to_string()); // SHOULD SUCCEED
    //test_hash(&server, &"YUIO".to_string()); // SHOULD FAIL

    //let dummy_socket = Rc::new(Socket{ip: "192.135.45.2".to_string(), port: 4444});
    
    //let mut key = server.create_session(dummy_socket.clone(), false);
    //print_key(&key);

    //key = server.create_session(dummy_socket.clone(), false);
    //print_key(&key);

    Server::poll(&server);
}
