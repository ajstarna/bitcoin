use std::collections::HashMap;

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


fn main() {
}
