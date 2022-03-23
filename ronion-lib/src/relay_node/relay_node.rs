use async_std::{net::{IpAddr, SocketAddr}, task};

pub struct RelayNode {
    ip: IpAddr,
    port: u16
}

impl RelayNode {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip: ip,
            port: port
        }
    }

    pub fn start(&self) {

    }

    async fn listen(&self, socket: SocketAddr) {
        let socket = SocketAddr::new(self.ip, self.port);
        let listen_future = self.listen(socket);
        
        task::block_on(listen_future); // bytte til async?
    }
}