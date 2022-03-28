use async_std::net::SocketAddr;

type RelayID = u32;

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Relay(RelayID),
    IP(SocketAddr),
    Current,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Relay {
    pub id: RelayID,
    pub addr: SocketAddr,
    pub pub_key: [u8; 32],
}

#[derive(PartialEq, Debug)]
pub enum ClientType {
    Consumer,
    Relay,
}

#[derive(PartialEq, Debug)]
pub struct HelloRequest {
    pub client_type: ClientType,
    pub public_key: [u8; 32],
}

#[derive(PartialEq, Debug)]
pub struct RelayPingRequest {
    pub port: u16,
    pub signing_public: [u8; 32],
}

#[derive(PartialEq, Debug)]
pub enum Message {
    HelloRequest(HelloRequest),
    HelloResponse([u8; 96]),

    Close(Option<String>),
    Payload(Vec<u8>),

    GetRelaysRequest(),
    GetRelaysResponse(Vec<Relay>),

    RelayPingRequest(RelayPingRequest),
    RelayPingResponse(),
}

#[derive(PartialEq, Debug)]
pub struct Onion {
    pub circuit_id: Option<u32>,
    pub message: Message,
    pub target: Target,
}
