use std::{env, net::IpAddr};

use core::index_node::index_node::IndexNode;

use ronion_index::key::{gen_keys, read_keypair};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|x| x == "gen-keys") {
        gen_keys().expect("unable to generate keys");
        return;
    }
    

    let keypair = read_keypair();
    let (ip, port) = parse_arguments(args);
    let node = IndexNode::new(ip, port, keypair);

    node.start();
}

fn parse_arguments(args: Vec<String>) -> (IpAddr, u16) {
    let addr: IpAddr = args[1].parse().unwrap();
    let port: u16 = args[2].parse().unwrap();

    (addr, port)
}
