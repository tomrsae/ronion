use async_std::net::TcpStream;

use super::circuit_connection::CircuitConnection;

pub struct Circuit {
    pub id: u32,
    pub incoming: CircuitConnection,
    pub outgoing: CircuitConnection
}

