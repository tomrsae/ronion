use std::{env, net::IpAddr};

use core::relay_node::relay_node::RelayNode;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (ip, port) = parse_arguments(args);
    let node = RelayNode::new(ip, port);

    node.register(index_addr, index_signing_pub_key);

    node.start();
}

fn parse_arguments(args: Vec<String>) -> (IpAddr, u16) {
    let addr: IpAddr = args[1].parse().unwrap();
    let port: u16 = args[2].parse().unwrap();

    (addr, port)
}
