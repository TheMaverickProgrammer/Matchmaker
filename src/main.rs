use rand::{distributions::Alphanumeric, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

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

    //
    // non mut fn
    //

    fn has_key(&self, key: &String) -> bool {
        self.sessions.iter().position(|session: &Session| session.key == *key) != None
    }

    fn has_socket(&self, socket: &Socket) -> bool {
        self.sessions.iter().position(|session: &Session| *session.socket == *socket) != None
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

    fn drop_session(&mut self, socket: &Socket) {
        if let Some(pos) = self.sessions.iter().position(|session: &Session| *session.socket == *socket) {
            self.sessions.remove(pos);
        }
    }
}

fn file_read_lines(path: &str) -> Vec<String> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let result = Vec::new();

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Ignore errors
        println!("{}. {}", index + 1, line);
    }

    result
}

fn print_key(key: &Option<String>) {
    match key {
        Some(x) => println!("Key was {}", x),
        None => println!("Empty key")
    }
}

fn main() {
    let client_hashes = file_read_lines("./hashes.txt");

    let mut server = Server::new(3000);
    let dummy_socket = Rc::new(Socket{ip: "192.135.45.2".to_string(), port: 4444});
    
    let mut key = server.create_session(dummy_socket.clone(), false);
    print_key(&key);

    key = server.create_session(dummy_socket.clone(), false);
    print_key(&key);
}
