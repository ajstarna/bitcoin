use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};
use sha2::{Sha256, Digest};
use std::mem;
use ethnum::U256;

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use super::transaction::{Hash, Transaction, Script, StackOp, TxOut, TxIn};

const BLOCK_HALVENING: u32 = 210_000; // after this many blocks, the block reward gets cut in half
const ORIGINAL_COINBASE: u32 = 21_000_000 * 50; // the number of Eves that get rewarded during the first halvening period (50 AdamCoin)
const STARTING_DIFFICULTY_BITS: DifficultyBits = DifficultyBits(0x1ec3a30c); // TODO: this is the "real" one --> 0x1d00ffff

/// This notation expresses the Proof-of-Work target as a coefficient/exponent format,
/// with the first two hexadecimal digits for the exponent and the next six hex digits as the coefficient.
/// target = coefficient * 2^(8*(exponent–3))
/// e.g. with bytes == 0x1903a30c:
/// target = 0x03a30c * 2^(0x08*(0x19-0x03)) = 0x0000000000000003A30C00000000000000000000000000000000000000000000
/// From the bitcoin wiki:
/// Note that this packed format contains a sign bit in the 24th bit, and for example the negation of the above target would be 0x1b8404cb in packed format.
/// Since targets are never negative in practice, however, this means the largest legal value for the lower 24 bits is 0x7fffff.
/// Additionally, 0x008000 is the smallest legal value for the lower 24 bits since targets are always stored with the lowest possible exponent
struct DifficultyBits (pub u32);

impl DifficultyBits {
    /// Convert the exponent representation into a vector of bytes that represents the actual number
    /// This can then be compared against a candidate block header hash to see if it fits the proof of work
    pub fn to_u256(&self) -> U256 {

	let exponent: u32 = ((self.0 >> 24) & 0xFF ).into(); // get the first byte. TODO: is the mask required?
	let base: u128 = (self.0 & 0xFFFFFF).into(); // next three bytes represent the base
	//vec![0, 10, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
	// convert to the special U256 type before doing the math

	println!("hello");
	println!("exponent = {:?}", exponent);
	println!("base = {:?}", base);	
	
	let base = U256::from_words(0, base);

	println!("============");
	let full_exponent: u32 = 8 * (exponent -  3);
	println!("full_exponent = {:?}", full_exponent);
	assert!(full_exponent < 255); // we can't be too big
	// TODO: the checked_pow() meethod didn't seem to return None when there was overflow hmmm
	let rhs = U256::from_words(0, 2).pow(full_exponent);
	println!("rhs = {:?}", rhs);	
	let difficulty_target = base * rhs;
	difficulty_target
	// U256::from_words(0x01_05_00_00_00_00_00_00_00_00_00_00_00_00_00_00, 0x00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00)
    }
}


struct BlockHeader {
    version: u32, // 4 bytes: A version number to track software/protocol upgrades
    previous_block_hash: Hash, // 32 bytes: A reference to the hash of the previous (parent) block in the chain
    merkle_root: Hash, // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
    time_stamp: u64, // 4  (8 is ok?) bytes: The approximate creation time of this block (in seconds elapsed since Unix Epoch)
    difficulty_bits: DifficultyBits, // 4 bytes: The Proof-of-Work algorithm difficulty target for this block
    nonce: Option<u32>, // 4 bytes: A counter used for the Proof-of-Work algorithm
}


impl BlockHeader {
    fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
	let (hi, low) = self.previous_block_hash.into_words();
	hasher.update(hi.to_be_bytes());
	hasher.update(low.to_be_bytes());
	let (hi, low) = self.merkle_root.into_words();
	hasher.update(hi.to_be_bytes());
	hasher.update(low.to_be_bytes());
        hasher.update(self.time_stamp.to_be_bytes());
        hasher.update(self.difficulty_bits.0.to_be_bytes());
	if let Some(nonce) = self.nonce {
            hasher.update(nonce.to_be_bytes());	    
	}
	let hash_vecs: Vec<u8> = hasher.finalize().to_vec();
	println!("hash_vecs = {:?}", hash_vecs);

	// we use a Cursor to read a Vec<u8> into two u128s, then store them inside a U256
	let mut rdr = Cursor::new(hash_vecs);
	let hi = rdr.read_u128::<BigEndian>().unwrap();
	let low = rdr.read_u128::<BigEndian>().unwrap();
        U256::from_words(hi, low)
    }
}


struct TransactionList {
    transactions: Vec<Transaction>,    
}
impl TransactionList {
    pub fn new(transactions: Vec<Transaction>) -> Self {
	Self { transactions}
    }
	
    pub fn get_merkle_root(&self) -> Hash {
	U256::ZERO
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

    /// We try multiple nonce values, each time hashing the block header wtih Sha256,
    /// once we have found a hash that satisfies the difficulty requirment, we return the block hash,
    /// with the appropriate nonce field set
    fn mine_block(&self, block_header:  &mut BlockHeader)  {
	let difficulty_target = block_header.difficulty_bits.to_u256(); // what we will compare our hashes against
	let mut nonce: u32 = 0;
	loop {
	    block_header.nonce = Some(nonce);
	    let struct_hash = block_header.hash();
	    println!("nonce = {:?}, hash = {:?}, leading_zeros = {:?}", nonce, struct_hash, struct_hash.leading_zeros());
	    //if struct_hash.is_less_than_or_equal(&difficulty_vector) {
	    if struct_hash <= difficulty_target {	    
		// we have found a difficult enough hash value, so we are done
		println!("Found a valid nonce for proof of work!");
		println!("hash as bytes = {:?}", struct_hash.to_be_bytes());		
		break;
	    }
	    nonce += 1;
	}
	println!("Difficulty target = {:?}", difficulty_target);
	println!("Difficulty target = {:?}", difficulty_target.to_be_bytes());	
	
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
	    previous_block_hash: U256::ZERO,
	    merkle_root: transaction_list.get_merkle_root(), // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
	    time_stamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), // now is after unix_epoch so we can unrwap
	    difficulty_bits: STARTING_DIFFICULTY_BITS,
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
    fn spawn_genesis() {
	let mut chain = BlockChain::new();
        assert_eq!(chain.height(), 0);
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();    
	chain.spawn_genesis_block(public_key);
        assert_eq!(chain.height(), 1);	
    }

    #[test]    
    fn target_repr_to_u256() {
	let difficulty_bits = DifficultyBits(0x1903a30c);
	let difficulty_target = difficulty_bits.to_u256();
	let answer_hi: u128 = 0x00_00_00_00_00_00_00_03_A3_0C_00_00_00_00_00_00;
	let answer_low: u128 = 0x00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00;
	let answer = U256::from_words(answer_hi, answer_low);
	assert_eq!(difficulty_target, answer);
    }

}
