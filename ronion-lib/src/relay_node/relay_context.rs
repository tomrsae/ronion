use super::circuit::Circuit;

pub struct RelayContext {
    pub circuits: Vec<Circuit>
}

impl RelayContext {
    pub fn new() -> Self {
        Self {
            circuits: Vec::new()
        }
    }

    pub fn get_circuit(&self, circuit_id: u32) -> Option<Circuit> {
        self.circuits.iter().find(|circuit| circuit.id == circuit_id).map(|circuit| *circuit)
    }
}