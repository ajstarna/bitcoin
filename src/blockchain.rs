use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use k256::{Secp256k1};
use ecdsa::{SigningKey, VerifyingKey};
use ecdsa::signature::{Signer, Verifier, Signature}; // trait in scope for signing a message

use crate::Hash;
use crate::script::{Script, StackOp, execute_scripts, hash_160_to_bytes};
use crate::transaction::{Transaction, TxOut, TxIn, TransactionError};
use crate::database::{TransactionDataBase};
use crate::block::{Block, DifficultyBits, BlockHeader, TransactionList};

const BLOCK_HALVENING: u32 = 210_000; // after this many blocks, the block reward gets cut in half
const ORIGINAL_COINBASE: u32 = 21_000_000 * 50; // the number of satoshis that get rewarded during the first halvening period (50 Bitcoin))
const STARTING_DIFFICULTY_BITS: DifficultyBits = DifficultyBits(0x1ec3a30c); // TODO: this is the "real" one --> 0x1d00ffff

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockChain {
    pub blocks: Vec<Block>, // TODO: move this to a DB. for now a vec should suffice. (How to handle forks though?)
    difficulty_bits: DifficultyBits,
    max_transactions_per_block: u32, // how many transactions can we fit in a block (TODO: actually a function of overall blocksize, not num transactions
    mempool: VecDeque<Transaction>, // the mempool is a queue of transactions that want to get added to a block (FIFO at least for now)
    transaction_database: TransactionDataBase, // keep track of previous transactions in an easier way. helps verify
}

impl BlockChain {

    pub fn new() -> Self {
	Self {
	    blocks: Vec::new(),
	    difficulty_bits: STARTING_DIFFICULTY_BITS, //TODO: change over time
	    max_transactions_per_block: 1000,
	    mempool: VecDeque::new(),
	    transaction_database: TransactionDataBase::new(),
	}
    }

    
    /// Is the blockchain empty/there are no blocks yets?
    /// Will mainly be called by the function that spawns the genesis block.
    /// We make it its own method so that if/when the data structure that holds the blockchain is changed,
    /// we have a modular location to check the length
    fn is_empty(&self) -> bool {
	self.len() == 0
    }

    /// return how many block are in the chain, i.e. the height
    fn len(&self) -> u32 {
	self.blocks.len() as u32
    }

    /// if the transaction is valid (the unlocking script unlocks the locking script),
    /// then it is adding to the mempool. else ag
    pub fn try_add_tx_to_mempool(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
	let mut tx_in_value_sum = 0; // the total value coming into this transaction from tx_ins
	for tx_in in &transaction.tx_ins {
	    // each tx_in must be unlocked
	    if let TxIn::TxPrevious {tx_hash, tx_out_index, unlocking_script, sequence  } = tx_in {
		let transaction_prev_opt = self.transaction_database.get(&tx_hash);
		if let Some(transaction_prev) = transaction_prev_opt {
                    // first check if the script actually unlocks it
		    let tx_out_to_unlock = &transaction_prev.tx_outs[*tx_out_index];                                        
		    let locking_script = &tx_out_to_unlock.locking_script;
		    let transaction_prev_hash = transaction_prev.hash_to_bytes();
		    let is_valid = execute_scripts(&unlocking_script, locking_script, &transaction_prev_hash);
                    if !is_valid {
                        return Err(TransactionError::InvalidScript);
                    }
                    // we unlocked it, so now and add to the total much we have to spend
		    tx_in_value_sum += tx_out_to_unlock.value;
		} else {
		    return Err(TransactionError::TxInNotFound);
		}
		// 
		
	    } else {
		// we can only take as inputs previous outputs. Only a miner may receive a coinbase reward.
		return Err(TransactionError::CoinbaseSpend);
	    }
	}

	// mext check that the tx_out values don't sum to more than the tx_in values
        let tx_out_value_sum = transaction.tx_outs.iter().fold(0, |sum, tx_out| sum + tx_out.value);
        
	if tx_out_value_sum > tx_in_value_sum {
	    return Err(TransactionError::OverSpend);
	}

	let _miner_tip = tx_in_value_sum - tx_out_value_sum; // todo: use this for priority in mempool?

	self.mempool.push_back(transaction);
	Ok(())
    }
    
    fn determine_coinbase_reward(&self) -> u32 {
	let num_halvenings = self.len() / BLOCK_HALVENING;
	let coinbase = ORIGINAL_COINBASE / (2 as u32).pow(num_halvenings);
	coinbase
    }

    fn construct_coinbase_transaction(&self, recipient: VerifyingKey<Secp256k1>) -> Transaction {
	let tx_in = TxIn::Coinbase {
	    coinbase: self.len(), // the coinbase field is sorta arbitrary, but adding the height here makes sure there won't be duplicate hashes of coinbase transactions,
	    sequence: 5580,
	};
	let reward = self.determine_coinbase_reward();

	// the locking script is the classic pay to public key of recipient.
        // TODO: make a function that gives us this script from the recipient        
	let public_key_bytes = recipient.to_encoded_point(true).to_bytes();
	let pub_hash = hash_160_to_bytes(&public_key_bytes);
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpHash160, StackOp::Bytes(pub_hash.into_boxed_slice()), StackOp::OpEqVerify, StackOp::OpCheckSig]};
        
	let tx_out = TxOut {
	    value: reward, // since there are no additional transaction fees this block, the tx_out is simply the entire reward
	    locking_script: locking_script,
	};
	Transaction {
	    version: 1,
	    lock_time: 100,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],	    
	}
    }
        
    /// given a new block, add it to the blockchain
    /// TODO: we should validate the block here or no? (yes, since the mined block could/would come from someone else)
    pub fn add_block(&mut self, block: Block) {
	self.blocks.push(block);
	self.transaction_database.read_blocks(&self.blocks);
        println!("added a block; current len = {:?}", self.len());        
    }

    /// given the recipient of the coinbase transaction, we construct and return a list of transactions to include in the
    /// next candidate block.
    /// The coinbase transaction is always the first in the list.
    fn construct_transaction_list(&mut self, recipient: VerifyingKey<Secp256k1>) -> TransactionList {
	let transaction = self.construct_coinbase_transaction(recipient);
	let mut transaction_list = TransactionList::new(vec![transaction]);
	if !self.is_empty() {
	    // if is_empty()< 1 (i.e. this is the genesis block), then do not go to the mempool
	    while self.mempool.len() > 0 && transaction_list.len() < self.max_transactions_per_block{
		let transaction = self.mempool.pop_front().unwrap(); // we already checked for len > 0, so can unwrap
		transaction_list.push(transaction);
	    }
	}
	transaction_list
    }

    /// get the hash of the block header of the previous block in the chain
    /// if the blockchain is empty, i.e. we are spawning the genesis block, then the previous hash is simply 0
    fn get_previous_block_hash(&self) -> Hash {
	if self.is_empty() {
	    Hash::zero()
	} else {
	    let previous_block = self.blocks.last().unwrap();
	    previous_block.block_header.hash()
	}
    }

    /// given the recipient of the coinbase reward, this method constructs a list of transactions from the mempool and returns a Block
    /// with the nonce value of the header initialized to None and pointing at the most recent block in the chain.
    /// The block can now be mined but adjusting the nonce and hashing
    pub fn construct_candidate_block(&mut self, recipient: VerifyingKey<Secp256k1>) -> Block {
	let transaction_list = self.construct_transaction_list(recipient);
	let previous_block_hash = self.get_previous_block_hash();
	let block_header = BlockHeader::new(
	    1,
	    previous_block_hash,
	    transaction_list.get_merkle_root(), // 32 bytes: A hash of the root of the merkle tree of this block’s transactions)
	    self.difficulty_bits
	);
	
	Block {
	    block_size: 100, // TODO: how is this measured?
	    block_header: block_header,
	    transaction_count: transaction_list.len(),
	    transaction_list: transaction_list,
	}
    }

    pub fn print_transactions(&self) {
        println!("{:?}", self.transaction_database);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn run_basic_blocks() {
	// a couple blocks here with only the coinbase transaction
	let mut chain = BlockChain::new();
	let num_blocks = 2;
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();    	
	for _ in 0..num_blocks {
	    let mut block = chain.construct_candidate_block(public_key);
	    block.mine();
	    println!("about to add block: {:?}", block);
	    chain.add_block(block);
	}
	
        assert_eq!(chain.len(), num_blocks);	
    }

    /// we attempt to add a transaction to the mempool that include a coinbase as a tx_in;
    /// this is invalid, since only the miner gets to construct a coinbase transaction
    #[test]    
    fn add_to_mempool_invalid_coinbase() {
	let mut chain = BlockChain::new();
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	let tx_out = TxOut {
	    value: 22,
	    locking_script: Script {ops: vec![StackOp::OpDup]},	    
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],		
	};

	
	let result = chain.try_add_tx_to_mempool(transaction);
        assert_eq!(result, Err(TransactionError::CoinbaseSpend));
    }

    /// we attempt to add a transaction to the mempool that include a reference to a tx that does not exist    
    #[test]
    fn add_to_mempool_invalid_missing_tx() {
	let mut chain = BlockChain::new();

	let transaction_hash = Hash::zero(); // this tx will not exist in the blockchain db
	
	let tx_in = TxIn::TxPrevious {
	    tx_hash: transaction_hash, 
	    tx_out_index: 0,
	    unlocking_script: Script{ops: vec![StackOp::OpDup]}, // arbitrary for this test
	    sequence: 1234,
	};
	// the tx_out is arbitrary
	let tx_out = TxOut {
	    value: 22,
	    locking_script: Script {ops: vec![StackOp::OpDup]},	    
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],		
	};

	let result = chain.try_add_tx_to_mempool(transaction);
        assert_eq!(result, Err(TransactionError::TxInNotFound));        
    }
    
    #[test]
    fn add_to_mempool_invalid_overpsend() {
	// we attempt to add a transaction to the mempool that wants to spend as tx outputs more than the tx ins
	// first we must mine an empty block to have a tx_out available to theoretically spend
	let mut chain = BlockChain::new();
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	let public_key_bytes = public_key.to_encoded_point(true).to_bytes();	
	let mut block = chain.construct_candidate_block(public_key);
	block.mine();
	chain.add_block(block);
	println!("{:?}", chain.transaction_database);

	// Note: this particular coinbase transaction has this hash.
	// A wallet would need to look it up by recipient public key or something like that
	// decimal: 38321321692519566122529587483305535719886798403229990577862410369545149044829
	// hex: 54B919753E5FC47F98A0574A2F5D5679726EAB8349DD943622C2DE14A497585D
	//let hi: u128 = 0x54_B9_19_75_3E_5F_C4_7F_98_A0_57_4A_2F_5D_56_79;
	//let low: u128 = 0x72_6E_AB_83_49_DD_94_36_22_C2_DE_14_A4_97_58_5D;
        let hash_bytes: [u8; 32] = [0x54, 0xB9, 0x19, 0x75, 0x3E, 0x5F, 0xC4, 0x7F, 0x98, 0xA0, 0x57, 0x4A, 0x2F, 0x5D, 0x56, 0x79,
                                    0x72, 0x6E, 0xAB, 0x83, 0x49, 0xDD, 0x94, 0x36, 0x22, 0xC2, 0xDE, 0x14, 0xA4, 0x97, 0x58, 0x5D];
	let transaction_hash = Hash::from(&hash_bytes);
	//let tx_hash_bytes = transaction_hash.to_be_bytes();
	
	let sig = private_key.try_sign(&hash_bytes).expect("should be able to sign the transaction hash here");
	let sig_as_bytes = sig.as_bytes().to_vec(); //TODO: is there a way to go right from &[u8] --> Box instead of through vec?
	let unlocking_script = Script {ops: vec![StackOp::Bytes(sig_as_bytes.into_boxed_slice()), StackOp::Bytes(public_key_bytes)]};

	
	let tx_in = TxIn::TxPrevious {
	    tx_hash: transaction_hash, // Hash of the transaction that we are getting this input from
	    tx_out_index: 0,// The index of the tx_out within the transaction (only one for the first block just the rewward to the miner)
	    unlocking_script: unlocking_script, 
	    sequence: 1234,
	};

	let tx_out = TxOut {
	    value: 1050000000 + 1, // 1 more than allowed
	    locking_script: Script {ops: vec![StackOp::OpDup]},	
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],		
	};

	
	let result = chain.try_add_tx_to_mempool(transaction);
        assert_eq!(result, Err(TransactionError::OverSpend));                
    }

    /// first we must mine an empty block to have a tx_out available to spend spend
    /// then we add to mempool a transaction that wants to spend some of the available funds
    /// We also check that the next candidate block will take the transaction from the mempool
    #[test]
    fn add_to_mempool_valid_spend() {
	let mut chain = BlockChain::new();
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	let public_key_bytes = public_key.to_encoded_point(true).to_bytes();	
	let mut block = chain.construct_candidate_block(public_key);
	block.mine();
	chain.add_block(block);
	println!("{:?}", chain.transaction_database);

	// Note: this particular coinbase transaction has this hash.
	// A wallet would need to look it up by recipient public key or something like that
	// decimal: 38321321692519566122529587483305535719886798403229990577862410369545149044829
	// hex: 54B919753E5FC47F98A0574A2F5D5679726EAB8349DD943622C2DE14A497585D
        let hash_bytes: [u8; 32] = [0x54, 0xB9, 0x19, 0x75, 0x3E, 0x5F, 0xC4, 0x7F, 0x98, 0xA0, 0x57, 0x4A, 0x2F, 0x5D, 0x56, 0x79,
                                    0x72, 0x6E, 0xAB, 0x83, 0x49, 0xDD, 0x94, 0x36, 0x22, 0xC2, 0xDE, 0x14, 0xA4, 0x97, 0x58, 0x5D];
	let transaction_hash = Hash::from(&hash_bytes);
	//let tx_hash_bytes = transaction_hash.to_be_bytes();
	
	let sig = private_key.try_sign(&hash_bytes).expect("should be able to sign the transaction hash here");
	let sig_as_bytes = sig.as_bytes().to_vec(); //TODO: is there a way to go right from &[u8] --> Box instead of through vec?
	let unlocking_script = Script {ops: vec![StackOp::Bytes(sig_as_bytes.into_boxed_slice()), StackOp::Bytes(public_key_bytes)]};

	
	let tx_in = TxIn::TxPrevious {
	    tx_hash: transaction_hash, // Hash of the transaction that we are getting this input from
	    tx_out_index: 0,// The index of the tx_out within the transaction (only one for the first block just the rewward to the miner)
	    unlocking_script: unlocking_script, 
	    sequence: 1234,
	};

	let tx_out = TxOut {
	    value: 1050000000 - 1, // 1 less than allowed (so tipping 1 Eve to the miner)
	    locking_script: Script {ops: vec![StackOp::OpDup]},	
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],		
	};

	
	let result = chain.try_add_tx_to_mempool(transaction);
	let expected = Ok(());
        assert_eq!(result, expected);
        assert_eq!(chain.mempool.len(), 1);

	// now see if when we construct a candidate block, it includes more than the coinbase transaction
	let second_block = chain.construct_candidate_block(public_key);
        assert_eq!(chain.mempool.len(), 0);		
	assert_eq!(second_block.transaction_count, 2);
	println!("{:?}", second_block);

	// and finally, a third block will only have the coinbase transaction yet again
	let third_block = chain.construct_candidate_block(public_key);
        assert_eq!(chain.mempool.len(), 0);		
	assert_eq!(third_block.transaction_count, 1);
	println!("{:?}", third_block);
	
    }
    
}
