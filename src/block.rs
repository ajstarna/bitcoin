use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};

use super::transaction::{Hash, Transaction, Script, StackOp, TxOut, TxIn};

const BLOCK_HALVENING: u32 = 210_000; // after this many blocks, the block reward gets cut in half
const ORIGINAL_COINBASE: u32 = 21_000_000 * 50; // the number of Eves that get rewarded during the first halvening period (50 AdamCoin)



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


struct BlockChain {
    blocks: Vec<Block>, // TODO: move this to a DB. for now a vec should suffice. (How to handle forks though?)
    height: u32,
}


impl BlockChain {

    fn new() -> Self {
	Self {
	    blocks: Vec::new(),
	    height: 0
	}
    }

    
    /// Is the blockchain empty/there are no blocks yets?
    /// Will mainly be called by the function that spawns the genesis block.
    /// We make it its own method so that if/when the data structure that holds the blockchain is changed,
    /// we have a modular location to check the length
    fn is_empty(&self) -> bool {
	self.blocks.len() == 0
    }


    fn construct_coinbase_tx_in(&self) -> TxIn {
	TxIn::Coinbase {
	    coinbase: 12345, // TODO: figure this out (sorta arbitrary i think?)
	    sequence: 5580,
	}
    }

    fn determine_coinbase_reward(&self) -> u32 {
	let num_halvenings = self.height / BLOCK_HALVENING;
	let coinbase = ORIGINAL_COINBASE / (2 as u32).pow(num_halvenings);
	coinbase
    }
    
    /// The first block in a blockchain, aka the "genesis block" needs to be created in a special way,
    /// since there is no previous block in this case
    fn spawn_genesis_block(&mut self) {
	assert!(self.is_empty()); // We can only spawn a genesis block when the blockchain is empty
	let tx_in = self.construct_coinbase_tx_in();
	let reward = self.determine_coinbase_reward();
	let tx_out = TxOut {
	    value: reward, // since there are no additional transaction fees this block, the tx_out is simply the entire reward
	    locking_script: Script {ops: vec![StackOp::PushVal(21)]},      // PushVerifyingKey(TEST_PUBLIC_KEY)]}
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 100,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],	    
	};
	
    }
}
