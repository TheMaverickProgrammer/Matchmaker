use crate::packets::{ClientPacket};

pub enum ThreadMessage {
    ClientPacket {
        socket_address: std::net::SocketAddr,
        id: u64,
        packet: ClientPacket
    }
}