struct IndexNode {
    ip: IpAddr,
    available_relays: Vec<_>, // Incomplete, need relay node struct or some other addressable tuple
}

impl IndexNode {
    pub fn new(ip: IpAddr) -> IndexNode {
        IndexNode {
            ip = ip,
            available_relays: Vec::new()
        }
    }

    fn listen_for_relay(&self) {

    }

    fn listen_for_consumer(&self) {

    }
}