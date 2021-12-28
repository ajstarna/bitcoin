use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};
use sha2::{Sha256, Digest};
use std::mem;

use super::transaction::{Hash, Transaction, Script, StackOp, TxOut, TxIn};

const BLOCK_HALVENING: u32 = 210_000; // after this many blocks, the block reward gets cut in half
const ORIGINAL_COINBASE: u32 = 21_000_000 * 50; // the number of Eves that get rewarded during the first halvening period (50 AdamCoin)

struct BlockHeader {
    version: u32, // 4 bytes: A version number to track software/protocol upgrades
    previous_block_hash: Hash, // 32 bytes: A reference to the hash of the previous (parent) block in the chain
    merkle_root: Hash, // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
    time_stamp: u64, // 4  (8 is ok?) bytes: The approximate creation time of this block (in seconds elapsed since Unix Epoch)
    difficulty_target: u32, // 4 bytes: The Proof-of-Work algorithm difficulty target for this block
    nonce: Option<u32>, // 4 bytes: A counter used for the Proof-of-Work algorithm
}

struct TransactionList {
    transactions: Vec<Transaction>,    
}
impl TransactionList {
    pub fn new(transactions: Vec<Transaction>) -> Self {
	Self { transactions}
    }
	
    pub fn get_merkle_root(&self) -> Hash {
	[0; 32]
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
}


impl BlockChain {

    fn new() -> Self {
	Self {
	    blocks: Vec::new(),
	}
    }

    
    /// Is the blockchain empty/there are no blocks yets?
    /// Will mainly be called by the function that spawns the genesis block.
    /// We make it its own method so that if/when the data structure that holds the blockchain is changed,
    /// we have a modular location to check the length
    fn is_empty(&self) -> bool {
	self.height() == 0
    }

    /// retusn how many block are in the chain, i.e. the height
    fn height(&self) -> u32 {
	self.blocks.len() as u32
    }

    fn construct_coinbase_tx_in(&self) -> TxIn {
	TxIn::Coinbase {
	    coinbase: 12345, // TODO: figure this out (sorta arbitrary i think?)
	    sequence: 5580,
	}
    }

    fn determine_coinbase_reward(&self) -> u32 {
	let num_halvenings = self.height() / BLOCK_HALVENING;
	let coinbase = ORIGINAL_COINBASE / (2 as u32).pow(num_halvenings);
	coinbase
    }

    fn mine_block(&self, block_header:  &mut BlockHeader)  {
	// create a Sha256 object
	let mut hasher = Sha256::new();

	// write input message
	hasher.update(b"hello world");
	
	// read hash digest and consume hasher
	let result = hasher.finalize();
	println!("result = {:?}", result);	
	//println!("result[..] = {:?}", result[..]);	
	//println!("result bytes = {:?}", result.as_bytes());	
	let a: Vec<u8> = result[..].to_vec();

	println!("a = {:?}", a);		
	//assert_eq!(result[..], hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")[..]);
    }


    /// given a new block, add it to the blockchain
    /// TODO: we should validate the block here or no?
    fn add_block(&mut self, block: Block) {
	self.blocks.push(block);
    }
		       
    /// The first block in a blockchain, aka the "genesis block" needs to be created in a special way,
    /// since there is no previous block in this case
    /// recipient is the public/verifying key of the person who will receive the coinbase for this block
    fn spawn_genesis_block(&mut self, recipient: VerifyingKey<Secp256k1>) {
	assert!(self.is_empty()); // We can only spawn a genesis block when the blockchain is empty
	// let now = SystemTime::now();
	let tx_in = self.construct_coinbase_tx_in();
	let reward = self.determine_coinbase_reward();
	let tx_out = TxOut {
	    value: reward, // since there are no additional transaction fees this block, the tx_out is simply the entire reward
	    locking_script: Script {ops: vec![StackOp::PushVerifyingKey(recipient)]},
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 100,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],	    
	};
	let transaction_list = TransactionList::new(vec![transaction]);
	let mut block_header =  BlockHeader {
	    version: 1, // 4 bytes: A version number to track software/protocol upgrades
	    previous_block_hash: [0; 32], 
	    merkle_root: transaction_list.get_merkle_root(), // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
	    time_stamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), // now is after unix_epoch so we can unrwap
	    difficulty_target: 2^30, // TODO
	    nonce: None, // this will get filled by the mining process
	};
	
	self.mine_block(&mut block_header);
	
	let genesis_block =  Block {
	    block_size: 100, // TODO: how is this measured?
	    block_header: block_header,
	    transaction_count: 1,
	    transaction_list: transaction_list,
	};
	
	self.add_block(genesis_block);	
    }
}

use std::time::{SystemTime};

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_spawn_genesis() {
	let mut chain = BlockChain::new();
        assert_eq!(chain.height(), 0);
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();    
	chain.spawn_genesis_block(public_key);
        assert_eq!(chain.height(), 1);	
    }

}
