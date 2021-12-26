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


fn main() {
    let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
    let TEST_PRIVATE_KEY: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
    let TEST_PUBLIC_KEY: VerifyingKey<Secp256k1> = TEST_PRIVATE_KEY.verifying_key();    
    println!("private = {:?}", TEST_PRIVATE_KEY);
    println!("public = {:?}", TEST_PUBLIC_KEY);
    println!("public.to_bytes = {:?}", TEST_PUBLIC_KEY.to_encoded_point(false));        
}
