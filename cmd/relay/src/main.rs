use std::{env, net::{IpAddr, SocketAddr}};

use core::relay_node::relay_node::RelayNode;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (ip, port, index_addr) = parse_arguments(args);
    let node = RelayNode::new(ip, port);

    node.register(index_addr, ronion_index::key::read_public());

    node.start();
}

fn parse_arguments(args: Vec<String>) -> (IpAddr, u16, SocketAddr) {
    let addr: IpAddr = args[1].parse().unwrap();
    let port: u16 = args[2].parse().unwrap();
    let index_addr = args[3].parse().unwrap();
    (addr, port, index_addr)
}
