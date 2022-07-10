use rustbreak::DeSerializer;
use serde::{Deserialize, Serialize};

/// Allows for the (de-)serialization with [pot].
#[derive(Debug, Clone, Copy, Default)]
pub struct Pot;

impl<T: for<'de> Deserialize<'de> + Serialize> DeSerializer<T> for Pot {
    fn serialize(&self, val: &T) -> rustbreak::error::DeSerResult<Vec<u8>> {
        // TODO use DeSerError::Other(anyhow::Error)
        let vec = pot::to_vec(val).expect("Fails hopefully not");
        Ok(vec)
    }

    fn deserialize<R: std::io::Read>(&self, s: R) -> rustbreak::error::DeSerResult<T> {
        // TODO use DeSerError::Other(anyhow::Error)
        let val = pot::from_reader(s).expect("Fails hopefully not");
        Ok(val)
    }
}
