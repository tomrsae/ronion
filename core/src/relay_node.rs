use async_std::net::IpAddr;

pub struct RelayNode {
    id: u32,
    ip: IpAddr
}

impl RelayNode {
    pub fn new(id: u32, ip: IpAddr) -> Self {
        RelayNode {
            id: id,
            ip: ip
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }
}