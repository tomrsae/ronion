use std::{env, net::IpAddr};

use core::index_node::index_node::IndexNode;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (ip, port) = parse_arguments(args);
    let node = IndexNode::new(ip, port);

    node.start();
}

fn parse_arguments(args: Vec<String>) -> (IpAddr, u16) {
    let addr: IpAddr = args[1].parse().unwrap();
    let port: u16 = args[2].parse().unwrap();

    (addr, port)
}
