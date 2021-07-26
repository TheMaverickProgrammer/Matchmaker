use rand::{distributions::Alphanumeric, Rng};
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
    sessions: Vec<Session>
}

impl Server {
    fn new(port: u16) -> Server {
        Server { port: port, sessions: Vec::new() }    
    }

    fn generate_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    fn has_key(&self, key: &String) -> bool {
        self.sessions.iter().position(|session: &Session| session.key == *key) != None
    }

    fn has_socket(&self, socket: &Socket) -> bool {
        self.sessions.iter().position(|session: &Session| *session.socket == *socket) != None
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

    fn drop_session(&mut self, session_key: &String) {
        if let Some(pos) = self.sessions.iter().position(|session: &Session| session.key == *session_key) {
            self.sessions.remove(pos);
        }
    }
}

fn print_key(key: &Option<String>) {
    match key {
        Some(x) => println!("Key was {}", x),
        None => println!("Empty key")
    }
}

fn main() {
    let mut server = Server::new(3000);
    let dummy_socket = Rc::new(Socket{ip: "192.135.45.2".to_string(), port: 4444});
    
    let mut key = server.create_session(dummy_socket.clone(), false);
    print_key(&key);

    key = server.create_session(dummy_socket.clone(), false);
    print_key(&key);
}
