use serde::{Serialize, Deserialize};
use ethereum_types::H256;

mod transaction;
mod script;
mod block;
pub mod blockchain;
mod database;

pub type Hash = H256;


