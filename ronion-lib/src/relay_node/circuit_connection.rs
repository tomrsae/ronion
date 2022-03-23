use async_std::net::TcpStream;

pub struct CircuitConnection {
    pub stream: TcpStream
}