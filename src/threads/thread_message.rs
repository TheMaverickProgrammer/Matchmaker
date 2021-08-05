use crate::packets::{ClientPacket};

pub enum ThreadMessage {
    Tick(Box<dyn FnOnce() + Send>),
    ClientPacket {
        socket_address: std::net::SocketAddr,
        id: u64,
        packet: ClientPacket
    }
}