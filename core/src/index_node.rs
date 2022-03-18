use async_std::{
    task,
    net::{IpAddr, SocketAddr, TcpListener, Incoming}
};

pub struct IndexNode {
    ip: IpAddr,
    available_relays: Vec<_> // Incomplete, blocked by relay node struct or some other addressable tuple
}

impl IndexNode {
    pub fn new(ip: IpAddr) -> IndexNode {
        IndexNode {
            ip: ip,
            available_relays: Vec::new()
        }
    }

    async fn listen_for_relay(&self, port: u16) {
        self.listen(port, |connection| async {
            // handle relay registration
        }).await
    }
    
    async fn listen_for_consumer(&self, port: u16) {
        self.listen(port, |connection| async {
            // handle consumer request
        }).await
    }
    
    async fn listen(&self, port: u16, connection_handler: dyn Fn(Incoming)) {
        let socket = SocketAddr::new(self.ip, port);
        let listener = TcpListener::bind(socket).await;
    
        for connection in listener.incoming() {
            task::spawn(connection_handler(connection)).expect("Failed to receive connection")
        }
    }
}