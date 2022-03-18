use crate::relay_node::RelayNode;

pub struct IndexContext {
    pub available_relays: Vec<RelayNode>
}

impl IndexContext {
    pub fn new() -> Self {
        IndexContext {
            available_relays: Vec::new()
        }
    }
}