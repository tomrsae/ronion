use async_std::net::IpAddr;

type RelayID = u32;

pub enum Target {
    Relay(RelayID),
    IP(IpAddr),
    Current,
}

pub struct Onion {
    target: Target,
    payload: Vec<u8>,
}

