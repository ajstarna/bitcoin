use std::collections::HashMap;

use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};

use crate::transaction::{Hash, TxOut};
use crate::block::{BlockChain};

/// This struct holds a mapping from utxo hash to the utxo for all exisitng blocks
/// It also keeps a record of how many blocks it has seen so far
struct UTXODataBase {
    utxos_by_hash: HashMap<Hash, TxOut>,
    num_blocks_analyzed: u32,
}

impl UTXODataBase {
    fn new() -> Self {
	UTXODataBase {
	    utxos_by_hash :HashMap::new(),
	    num_blocks_analyzed: 0
	}
    }

    /// given a blockchain, we reads blocks that we have not already read yet, and include the
    /// TxOuts from the newly read blocks into our storage
    fn read_blocks(&self, blockchain: BlockChain) {
	for block in blockchain.iter().skip(self.num_blocks_analyzed) {
	    for transaction in block.transaction_list {
		println!("transaction = {:?}", transaction);
	    }
	    self.num_blocks_analyzed += 1;
	}
    }
}



/// eventually i think the blocks will be stored differently than a Vec<Block>
/// TODO
struct BlockDataBase {}
struct BlockHeaderDataBase {}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn read_blocks() {
	let mut chain = BlockChain::new();	
	let num_blocks = 3;
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	// add some aribtrary blocks to the chain (don't bother mining them)
	for _ in 0..num_blocks {
	    let mut block = chain.construct_candidate_block(public_key);
	    println!("about to add block: {:?}", block);
	    chain.add_block(block);
	}

	let utxo_database = UTXODataBase::new();
	utxo_database.read_blocks(chain);
    }
