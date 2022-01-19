use std::collections::HashMap;

use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};

use crate::transaction::{Transaction, TxOut};
use crate::{Hash};
use crate::block::{BlockChain};

/// This struct holds a mapping from transaction hash to the transaction for all exisitng blocks
/// It also keeps a record of how many blocks it has seen so far
#[derive(Debug)]
pub struct TransactionDataBase {
    transactions_by_hash: HashMap<Hash, Transaction>,
    num_blocks_analyzed: u32,
}

impl TransactionDataBase {
    pub fn new() -> Self {
	// TODO: can i use Self on the next line?
	TransactionDataBase {
	    transactions_by_hash :HashMap::new(),
	    num_blocks_analyzed: 0
	}
    }

    /// given a blockchain, we reads blocks that we have not already read yet, and include the
    /// TxOuts from the newly read blocks into our storage
    pub fn read_blocks(&mut self, blockchain: BlockChain) {
	// TODO: impl iterator for blockchain struct itself?
	for block in blockchain.blocks.into_iter().skip(self.num_blocks_analyzed as usize) {
	    for transaction in block.transaction_list.transactions {
		println!("transaction = {:?}", transaction);
		let transaction_hash = transaction.hash();
		println!("transaction_hash = {:?}", transaction_hash);
		self.transactions_by_hash.insert(transaction_hash, transaction);
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
	    //let current_hash = 
	}

	let mut database = TransactionDataBase::new();
	database.read_blocks(chain);
	assert_eq!(database.num_blocks_analyzed, num_blocks);
	// each block only has the coinbase transaction
	assert_eq!(database.transactions_by_hash.len(), num_blocks as usize);	
    }
}
