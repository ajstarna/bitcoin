use ethereum_types::U256;

mod transaction;
mod script;
mod block;
pub mod blockchain;
mod database;
mod merkle;

pub type Hash = U256;
