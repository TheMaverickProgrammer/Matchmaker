use std::net::{UdpSocket, SocketAddr};
use crate::threads::clock_thread::TICK_RATE;
use num_derive;

// enums
#[derive(num_derive::FromPrimitive)]
enum PacketId {
    PingPong = 0,
    Ack = 1,
    Create = 2,
    Join = 3 ,
    Close = 4,
    Error = 5
}

pub enum ServerPacket<'a> {
    Pong,
    Ack {
        id: u64
    },
    Create {
        session_key: &'a str
    },
    Join {
        client_addr: &'a SocketAddr
    },
    Close,
    Error {
        id: u64,
        message: &'a str
    }
}

#[derive(Debug)]
pub enum ClientPacket {
    Ping,
    Ack {
        id: u64
    },
    Create {
        client_hash: String,
        password_protected: bool
    },
    Join {
        client_hash: String,
        session_key: String
    },
    Close
}

// packets

pub struct Packet {
    pub id: u64,
    pub creation_time: std::time::Instant,
    pub data: Vec<u8>
}

// Clients have PacketShippers

pub struct PacketShipper {
    socket_address: SocketAddr,
    next_id: u64,
    backed_up: Vec<Packet>
}

impl PacketShipper {
    pub fn new(socket_address: SocketAddr) -> PacketShipper {
        PacketShipper {
            socket_address,
            next_id: 0,
            backed_up: Vec::new()
        }
    }

    pub fn send(&mut self, socket: &UdpSocket, packet: &ServerPacket) {
        let mut data = vec![];

        write_u64(&mut data, self.next_id);
        data.extend(build_server_packet(packet));

        let _ = socket.send_to(&data, self.socket_address);

        self.backed_up.push(Packet {
            id: self.next_id,
            creation_time: std::time::Instant::now(),
            data
        });

        self.next_id += 1;
    }

    #[allow(dead_code)]
    pub fn resend_unacknowledged_packets(&self, socket: &UdpSocket) {
        let retry_delay = std::time::Duration::from_secs_f64(1.0 / TICK_RATE);

        let iter = self
            .backed_up
            .iter()
            .take_while(|packet| packet.creation_time.elapsed() >= retry_delay);

        for packet in iter {
            let buf = &packet.data;

            if socket.send_to(buf, self.socket_address).is_err() {
                // socket buffer is probably full
                break;
            }
        }
    }

    pub fn acknowledge(&mut self, id: u64) {
        self
        .backed_up
        .iter()
        .position(|packet| packet.id == id)
        .map(|position| self.backed_up.remove(position));
    }
}

// Clients have PacketRecievers

struct RecievedPacket {
    pub id: u64,
    pub packet: ClientPacket
}

#[allow(dead_code)]
pub struct PacketReciever {
    socket_address: std::net::SocketAddr,
    next_id: u64,
    backed_up: Vec<RecievedPacket>,
    last_message_time: std::time::Instant
}

impl PacketReciever {
    pub fn new(socket_address: std::net::SocketAddr) -> PacketReciever {
        PacketReciever {
            socket_address,
            next_id: 0,
            backed_up: Vec::new(),
            last_message_time: std::time::Instant::now()
        }
    }

    pub fn get_last_message_time(&self) -> &std::time::Instant {
        &self.last_message_time
    }

    pub fn sort_packets(&mut self,
        socket: &UdpSocket,
        id: u64,
        packet: ClientPacket
    ) -> Option<ClientPacket> {
        self.last_message_time = std::time::Instant::now();

        self.send_ack(socket, id);

        if id < self.next_id {
            // ignore old packets
            None
        } else {
            self.next_id = id + 1;
            Some(packet)
        }
    }

    fn send_ack(&self, socket: &UdpSocket, id: u64) {
        let buf = build_server_packet(&ServerPacket::Ack {
            id
        });

        let _ = socket.send_to(&buf, self.socket_address);
    }
}

// reader 

pub fn read_byte(buf: &mut &[u8]) -> Option<u8> {
    if buf.is_empty() {
        return None;
    }

    let byte = buf[0];

    *buf = &buf[1..];

    Some(byte)
}

pub fn read_bool(buf: &mut &[u8]) -> Option<bool> {
    read_byte(buf).map(|byte| byte != 0)
}

pub fn read_u16(buf: &mut &[u8]) -> Option<u16> {
    use byteorder::{ByteOrder, LittleEndian};

    if buf.len() < 2 {
        *buf = &buf[buf.len()..];
        return None;
    }

    let data = LittleEndian::read_u16(buf);

    *buf = &buf[2..];

    Some(data)
}

#[allow(dead_code)]
pub fn read_u32(buf: &mut &[u8]) -> Option<u32> {
    use byteorder::{ByteOrder, LittleEndian};

    if buf.len() < 4 {
        *buf = &buf[buf.len()..];
        return None;
    }

    let data = LittleEndian::read_u32(buf);

    *buf = &buf[4..];

    Some(data)
}

pub fn read_u64(buf: &mut &[u8]) -> Option<u64> {
    use byteorder::{ByteOrder, LittleEndian};

    if buf.len() < 8 {
        *buf = &buf[buf.len()..];
        return None;
    }

    let data = LittleEndian::read_u64(buf);

    *buf = &buf[8..];

    Some(data)
}

pub fn read_string_u8(buf: &mut &[u8]) -> Option<String> {
    let len = read_byte(buf)? as usize;
    read_string(buf, len)
}

fn read_string(buf: &mut &[u8], len: usize) -> Option<String> {
    if buf.len() < len {
        *buf = &buf[buf.len()..];
        return None;
    }

    let string_slice = std::str::from_utf8(&buf[..len]).ok();

    *buf = &buf[len..];

    let string = String::from(string_slice?);

    Some(string)
}

fn parse_headers(buf: &mut &[u8]) -> Option<u64> {
    Some(read_u64(buf)?)
}

fn parse_packet(buf: &mut &[u8]) -> Option<ClientPacket> {
    let packet_type = num::FromPrimitive::from_u16(read_u16(buf)?);

    match packet_type {
        Some(PacketId::PingPong) => Some(ClientPacket::Ping),
        Some(PacketId::Ack) => Some(ClientPacket::Ack {
            id: read_u64(buf)?
        }),
        Some(PacketId::Create) => Some(ClientPacket::Create {
            client_hash: read_string_u8(buf)?,
            password_protected: read_bool(buf)?
        }),
        Some(PacketId::Join) => Some(ClientPacket::Join{
            client_hash: read_string_u8(buf)?,
            session_key: read_string_u8(buf)?
        }),
        Some(PacketId::Close) => Some(ClientPacket::Close),
        _ => None
    }
}

// writers

#[allow(dead_code)]
pub fn write_bool(buf: &mut Vec<u8>, data: bool) {
    buf.push(if data { 1 } else { 0 });
}

pub fn write_u16(buf: &mut Vec<u8>, data: u16) {
    use byteorder::{ByteOrder, LittleEndian};

    let mut buf_16 = [0u8; 2];
    LittleEndian::write_u16(&mut buf_16, data);
    buf.extend(&buf_16);
}

#[allow(dead_code)]
pub fn write_u32(buf: &mut Vec<u8>, data: u32) {
    use byteorder::{ByteOrder, LittleEndian};

    let mut buf_32 = [0u8; 4];
    LittleEndian::write_u32(&mut buf_32, data);
    buf.extend(&buf_32);
}

pub fn write_u64(buf: &mut Vec<u8>, data: u64) {
    use byteorder::{ByteOrder, LittleEndian};

    let mut buf_64 = [0u8; 8];
    LittleEndian::write_u64(&mut buf_64, data);
    buf.extend(&buf_64);
}

pub fn write_string_u8(buf: &mut Vec<u8>, data: &str) {
    let len = if data.len() < u8::MAX.into() {
        data.len() as u8
    } else {
        u8::MAX
    };

    buf.push(len);
    buf.extend(&data.as_bytes()[0..len.into()]);
}

pub fn build_server_packet(packet: &ServerPacket) -> Vec<u8> {
    let mut vec = Vec::new();
    let buf = &mut vec;

    match packet {
        ServerPacket::Pong => {
            write_u16(buf, PacketId::PingPong as u16);
        },
        ServerPacket::Ack { id } => {
            write_u16(buf, PacketId::Ack as u16);
            write_u64(buf, *id);
        },
        ServerPacket::Create { session_key } => {
            write_u16(buf, PacketId::Create as u16);
            write_string_u8(buf, *session_key);
        },
        ServerPacket::Join { client_addr } => {
            write_u16(buf, PacketId::Join as u16);
            write_string_u8(buf, &client_addr.to_string());
        },
        ServerPacket::Close => {
            write_u16(buf, PacketId::Close as u16);
        },
        ServerPacket::Error { id, message } => {
            write_u16(buf, PacketId::Error as u16);
            write_u64(buf, *id);
            write_string_u8(buf, *message);
        }
    }

    vec
}

pub fn parse_client_packet(mut buf: &[u8]) -> Option<(u64, ClientPacket)> {
    Some((parse_headers(&mut buf)?, parse_packet(&mut buf)?))
}