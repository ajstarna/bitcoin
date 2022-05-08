use serde::{Serialize, Deserialize};
use serde_with::{serde_as}; // needed for a hashmap of a remote type
use std::collections::HashMap;

use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};

use crate::transaction::{Transaction, TxOut};
use crate::block::{Block};
use crate::{Hash, HashDef};
use crate::blockchain::{BlockChain};

/// This struct holds a mapping from transaction hash to the transaction for all exisitng blocks
/// It also keeps a record of how many blocks it has seen so far
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDataBase {
    #[serde_as(as = "HashMap<HashDef, Transaction>")]    
    transactions_by_hash: HashMap<Hash, Transaction>,
    num_blocks_analyzed: u32,
}

impl TransactionDataBase {
    pub fn new() -> Self {
	Self {
	    transactions_by_hash :HashMap::new(),
	    num_blocks_analyzed: 0
	}
    }

    pub fn get(&self, entry: &Hash) -> Option<&Transaction> {
	self.transactions_by_hash.get(entry)
    }
	
    /// given a blockchain, we read blocks that we have not already read yet, and include the
    /// TxOuts from the newly read blocks into our storage
    pub fn read_blocks(&mut self, blocks: &Vec<Block>) {
	// TODO: impl iterator for blockchain struct itself?
	for block in blocks.into_iter().skip(self.num_blocks_analyzed as usize) {
	    for transaction in &block.transaction_list.transactions {
		println!("transaction = {:?}", transaction);
		let transaction_hash = transaction.hash();
		println!("transaction_hash = {:?}", transaction_hash);
		self.transactions_by_hash.insert(transaction_hash, transaction.clone());
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
    fn test_read_blocks() {
	let mut chain = BlockChain::new();	
	let num_blocks = 3;
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	// add some aribtrary blocks to the chain (don't bother mining them)
	//let hashes_vec = Vec::new();
	for _ in 0..num_blocks {
	    let block = chain.construct_candidate_block(public_key);
	    println!("about to add block: {:?}", block);
	    chain.add_block(block);
	}

	let mut database = TransactionDataBase::new();
	database.read_blocks(&chain.blocks);
	assert_eq!(database.num_blocks_analyzed, num_blocks);
	// each block only has the coinbase transaction
	assert_eq!(database.transactions_by_hash.len(), num_blocks as usize);	
    }
}
