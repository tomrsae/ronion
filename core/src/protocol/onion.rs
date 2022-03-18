use async_std::net::IpAddr;

type RelayID = u32;

#[derive(Clone)]
pub enum Target {
    Relay(RelayID),
    IP(IpAddr),
    Current,
}

pub struct Onion {
    pub target: Target,
    pub payload: Vec<u8>,
}
