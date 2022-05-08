use serde::{Serialize, Deserialize};
use ethnum::U256;

mod transaction;
mod script;
mod block;
pub mod blockchain;
mod database;


#[derive(Serialize, Deserialize)]
#[serde(remote = "U256")]
pub struct HashDef(pub [u128; 2]);

pub type Hash = U256;


