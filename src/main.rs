// TODO: I forgot to always send the valid client hash with the packets

use rand::{distributions::Alphanumeric, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::collections::HashMap;
use std::net;
use std::env;
use std::time::Instant;

use crate::packets;
use crate::threads::{create_listening_thread, ThreadMessage};

struct Session {
    key: String,
    password_protected: bool
}

#[derive(PartialEq)]
struct Client {
    reciever: PacketReciever,
    shipper: PacketShipper,
    session: Option<Session>
}

struct Server {
    port: u16,
    clients: HashMap<SocketAddr, Client>,
    sessions: HashMap<String, Client>,
    valid_client_hashes: Vec<String>
}

impl Server {

    //
    // static fn
    //

    pub fn new(port: u16) -> Server {
        Server { 
            port: port, 
            clients: HashMap::new(),
            sessions: HashMap::new(),
            valid_client_hashes: Vec::new(), 
        }
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

        let(tx, rx) = mpsc::channel();
        create_listening_thread(tx, socket.try_clone()?);

        println!("Server started");

        let mut time = Instant::now();
        let mut last_ping_pong = Instant::now();

        loop {
            match rx.recv()? {
                ThreadMessage::ClientPacket {
                    socket_address,
                    id,
                    packet
                } => {
                    if self.has_socket(socket_address) {
                        let mut reciever = self.clients.get(socket_address).unwrap().reciever;
                        
                        if Some(data) = reciever.sort_packets(socket, id, packet) {
                            self.handle_packet(&socket, socket_address, packet)
                        }
                    } else {
                        // new connection
                        let client = Client { 
                            reciever: PacketReciever::new(socket_address),
                            shipper: PacketShipper::new(socket_address),
                            session: None
                        };
    
                        let mut reciever = client.reciever;

                        if Some(data) = reciever.sort_packets(socket, id, packet) {
                            self.handle_packet(&socket, socket_address, packet)
                        }

                        self.clients.insert(socket_address, client);
                    }
                }
            }
        }
    }

    fn handle_packet(&mut self, socket: &UdpSocket, socket_address: SocketAddr, packet: ClientPacket) {
        match packet {
            ClientPacket::Ping => {
                // TODO: send pong and update heartbeat for this client
            },
            ClientPacket::Ack { id } => {
                // TODO: handle internal acks...
            },
            ClientPacket::Create { password_protected } => {
                if let Some(key) = self.create_session(socket_addr, password_protected) {
                    // TODO: send key
                } else {
                    // TODO: send error packet
                }
            },
            ClientPacket::Join { session_key } => {
                // TODO: find session and then swap socket addresses
                //       then drop session from availability
            },
            ClientPacket::Close => {
                self.drop_client_session(socket_addr);
            }
        }
    }

    //
    // non mut fn
    //

    fn has_key(&self, key: &str) -> bool {
        self.sessions.contains_key(key)
    }

    fn has_socket(&self, socket_address: &SocketAddr) -> bool {
        self.clients.contains_key(socket_address)
    }

    fn valid_client_hash(&self, hash: &str) -> bool {
        self.valid_client_hashes.iter().position(|h: &String| *h == *hash) != None
    }

    //
    // mut fn
    //

    pub fn support_client_hashes(&mut self, hashes: Vec<String>) {
        self.valid_client_hashes = hashes;
    }

    fn create_session(&mut self, socket_address: &SocketAddr, password_protected: bool) -> Option<String> {
        let mut result = None;

        if !self.has_socket(&socket_address) {
            loop {
                let new_key = Server::generate_key();

                if !self.has_key(&new_key) {
                    let client = Client { 
                        reciever: PacketReciever::new(socket_address),
                        shipper: PacketShipper::new(socket_address),
                        session: Some(Session { key: new_key.clone(), password_protected: password_protected })
                    };

                    self.clients.insert(socket_address, client);
                    self.sessions.insert(new_key, client);

                    println!("Session created for client {}:{} with key {} (password_protected: {})", socket_address.addr.ip(), socket_address.addr.port(), new_key, password_protected);
                    result = Some(new_key);
                    break;
                }
            }
        } else {
            println!("Session for {}:{} cannot be created because it already exists", socket_address.addr.ip(), socket_address.addr.port());
        }

        result
    }
    
    fn get_client_from_session(&mut self, key: &str) -> Option<&Client> {
        self.sessions.get(key)
    }

    fn get_client_from_open_session(&mut self) -> Option<&Client> {
        self.sessions
            .into_values()
            .find(|client: &&Client| client.session.unwrap().password_protected == false)
    }

    fn drop_client_session(&mut self, socket_address: &std::net::SocketAddr) -> bool {
        if let Some(&client) = self.clients.remove(socket_address) {
            if let Some(session) = client.session {
                self.sessions.remove(session);
            }

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

    Server::poll(&mut server);
}
