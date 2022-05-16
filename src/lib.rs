use ethereum_types::U256;

mod transaction;
mod script;
mod block;
pub mod blockchain;
mod database;
mod merkle;
mod mempool;
pub type Hash = U256;

/// This trait defines a function that returns a hash created by
/// running the data through SHA256 twice
/// Needed example for creating the merkle root
pub trait DoubleSHA {
    fn sha256d(&self) -> Hash;
}
