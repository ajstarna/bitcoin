#![allow(dead_code)] // TODO: get rid
#![allow(unused_variables)] // TODO: get rid

use std::collections::HashMap;

use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};

mod transaction;
mod block;

use transaction::{Hash, TxOut};

/// This struct holds a mapping from utxo hash to the utxo for all exisitng blocks
/// It also keeps a record of how many blocks it has seen so far
/// Note: this is not ~fundamental to the blockchain itself, but necessary for validation
struct UTXODataBase {
    utxos_by_hash: HashMap<Hash, TxOut>,
}

impl UTXODataBase {
    fn new() -> Self {
	UTXODataBase { utxos_by_hash : HashMap::new() }
    }
}

