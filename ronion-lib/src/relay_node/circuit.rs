use async_std::io::Result;

use super::circuit_connection::CircuitConnection;

pub struct Circuit {
    pub id: u32,
    pub incoming: CircuitConnection,
    pub outgoing: CircuitConnection
}

impl Circuit {
    pub fn activate(&self) -> Result<()> {
        

        Ok(())
    }

    async fn activate_internal() -> Result<()> {
        Ok(())
    }
}