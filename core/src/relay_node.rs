use async_std::net::IpAddr;

pub struct RelayNode {
    pub id: u32,
    pub ip: IpAddr
}

impl RelayNode {
    pub fn new(id: u32, ip: IpAddr) -> Self {
        RelayNode {
            id: id,
            ip: ip
        }
    }
}