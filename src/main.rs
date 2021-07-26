use rand::{distributions::Alphanumeric, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::net;
use std::env;
use std::time::Instant;

#[derive(PartialEq)]
struct Socket {
    addr: std::net::SocketAddr
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

    pub fn new(port: u16) -> Server {
        Server { port: port, sessions: Vec::new(), client_hashes: Vec::new() }
    }

    fn generate_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    fn is_password_command(token: &str) -> bool {
        token == "PASSWORD-ONLY"
    }

    pub fn poll(server: &mut Server) {
        let ipaddr = "0.0.0.0".to_string() + ":" + &server.port.to_string();
        let socket = net::UdpSocket::bind(ipaddr).expect("Failed to bind host socket");
        let mut buf = [0u8; 100];
        let mut last_packet_tm = Instant::now();

        loop {
            let mut read = true;

            while read {
                match socket.recv_from(&mut buf) {
                    Ok((bytes, client_addr)) => {
                        let packet = std::str::from_utf8(&mut buf[0..bytes]).unwrap();

                        println!("client {} sent bytes {}: \"{}\"", client_addr, bytes, packet);

                        let tokens: Vec<&str> = packet.split_whitespace().collect();
                        
                        if tokens.len() > 1 && server.valid_client_hash(tokens[0]) {
                            match tokens[1] {
                                "CREATE" => {
                                    let new_user = Rc::<Socket>::new(Socket{addr: client_addr});
                                    let mut password_protected = false;

                                    if tokens.len() > 2 {
                                        password_protected = Server::is_password_command(tokens[2]);
                                    }

                                    match server.create_session(new_user.clone(), password_protected) {
                                        Some(session_key) => {
                                            match socket.send_to(&session_key.into_bytes(), &new_user.addr) {
                                                Ok(bytes) => {
                                                    println!("{} bytes sent to client", bytes);
                                                },
                                                Err(e) => {
                                                    println!("Error creating session: {}", e);
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                "JOIN" => {
                                    let mut other_addr: Option<std::net::SocketAddr> = None;

                                    if tokens.len() > 2 {
                                        let session_key = tokens[2];

                                        match server.get_session(session_key) {
                                            Some(session) => {
                                                // [1] reply with the ip address
                                                match socket.send_to(&session.socket.addr.to_string().into_bytes(), &client_addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to client", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }

                                                // [2] tell the session creator there is a match
                                                match socket.send_to(&client_addr.to_string().into_bytes(), &session.socket.addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to session owner", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }

                                                // [3] mark the session for a drop
                                                other_addr = Some(session.socket.addr.clone());
                                            },
                                            None => {
                                                match socket.send_to(&"ERROR".as_bytes(), &client_addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to client", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    } 
                                    else {
                                        match server.get_open_session() {
                                            Some(session) => {
                                                // [1] reply with the ip address
                                                match socket.send_to(&session.socket.addr.to_string().into_bytes(), &client_addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to client", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }

                                                // [2] tell the session creator there is a match
                                                match socket.send_to(&client_addr.to_string().into_bytes(), &session.socket.addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to session owner", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }

                                                // [3] mark the session for a drop
                                                other_addr = Some(session.socket.addr.clone());
                                            },
                                            None => {
                                                match socket.send_to(&"ERROR".as_bytes(), &client_addr) {
                                                    Ok(bytes) => {
                                                        println!("{} bytes sent to client", bytes);
                                                    },
                                                    Err(e) => {
                                                        println!("Error joining session: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // If we found a match, drop the session from the waiting pool
                                    if let Some(addr) = other_addr {
                                        server.drop_session(&addr);
                                    }
                                },
                                "CLOSE" => {
                                    if server.drop_session(&client_addr) {
                                        match socket.send_to(&"OK".as_bytes(), &client_addr) {
                                            Ok(bytes) => {
                                                println!("{} bytes sent to client", bytes);
                                            },
                                            Err(e) => {
                                                println!("Error dropping session: {}", e);
                                            }
                                        }
                                    } else {
                                        match socket.send_to(&"ERROR".as_bytes(), &client_addr) {
                                            Ok(bytes) => {
                                                println!("{} bytes sent to client", bytes);
                                            },
                                            Err(e) => {
                                                println!("Error dropping session: {}", e);
                                            }
                                        }
                                    }
                                },
                                e => {
                                    println!("Unrecognized command {} !", e);
                                }
                            }
                        }

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

    fn has_key(&self, key: &str) -> bool {
        self.sessions.iter().position(|session: &Session| session.key == *key) != None
    }

    fn has_socket(&self, socket: &Socket) -> bool {
        self.sessions.iter().position(|session: &Session| *session.socket == *socket) != None
    }

    fn valid_client_hash(&self, hash: &str) -> bool {
        self.client_hashes.iter().position(|h: &String| *h == *hash) != None
    }

    //
    // mut fn
    //

    pub fn support_client_hashes(&mut self, hashes: Vec<String>) {
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
                    println!("Session created for client {}:{} with key {} (password_protected: {})", socket.addr.ip(), socket.addr.port(), new_key, password_protected);
                    result = Some(new_key);
                    break;
                }
            }
        } else {
            println!("Session for {}:{} cannot be created because it already exists", socket.addr.ip(), socket.addr.port());
        }

        result
    }
    
    fn get_session(&mut self, key: &str) -> Option<&Session> {
        self.sessions.iter().find(|session: &&Session| session.key == *key)
    }

    fn get_open_session(&mut self) -> Option<&Session> {
        self.sessions.iter().find(|session: &&Session| session.password_protected == false)
    }

    fn drop_session(&mut self, socket_addr: &std::net::SocketAddr) -> bool {
        if let Some(pos) = self.sessions.iter().position(|session: &Session| session.socket.addr == *socket_addr) {
            self.sessions.remove(pos);
            return true;
        }

        false
    }
}

//
// util fn
//

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

//
// entry
// 

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

    //let dummy_socket = Rc::new(Socket{addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)});
    
    //let mut key = server.create_session(dummy_socket.clone(), false);
    //print_key(&key);

    //key = server.create_session(dummy_socket.clone(), false);
    //print_key(&key);

    Server::poll(&mut server);
}
