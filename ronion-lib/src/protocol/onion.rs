use async_std::net::{SocketAddr};

type RelayID = u32;

#[derive(Clone)]
pub enum Target {
    Relay(RelayID),
    IP(SocketAddr),
    Current,
}

pub struct Relay {
    pub id: RelayID,
    pub addr: SocketAddr
}

pub enum Message {
    HelloRequest([u8; 32]),
    HelloResponse(),

    Close(Option<String>),
    Payload(Vec<u8>),

    GetRelaysRequest(),
    GetRelaysResponse(Vec<Relay>),
 
    RelayPingRequest(),
    RelayPingResponse(),
}


pub struct Onion {
    pub target: Target,
    pub circuid_id: Option<u32>,
    pub message: Message,
}
