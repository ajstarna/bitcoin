
mod transaction;
use transaction::{Hash, Transaction};

struct BlockHeader {
    version: u32, // 4 bytes: A version number to track software/protocol upgrades
    previous_block_hash: Hash, // 32 bytes: A reference to the hash of the previous (parent) block in the chain
    merkle_root: Hash, // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
    time_stamp: u32, // 4 bytes: The approximate creation time of this block (in seconds elapsed since Unix Epoch)
    difficulty_target: u32, // 4 bytes: The Proof-of-Work algorithm difficulty target for this block
    nonce: u32, // 4 bytes: A counter used for the Proof-of-Work algorithm
}

struct TransactionList {
    transactions: Vec<Transaction>,    
}
impl TransactionList {
    pub fn get_merkle_root(self) -> Hash {
	5
    }
}

struct Block {
    block_size: u32,
    block_header: BlockHeader,
    transaction_count: u32,
    transaction_list: TransactionList,
    
}


