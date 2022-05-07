use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use ethnum::U256;

use std::time::{SystemTime};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::Hash;
use crate::transaction::{Transaction};

/// This notation expresses the Proof-of-Work target as a coefficient/exponent format,
/// with the first two hexadecimal digits for the exponent and the next six hex digits as the coefficient.
/// target = coefficient * 2^(8*(exponent–3))
/// e.g. with bytes == 0x1903a30c:
/// target = 0x03a30c * 2^(0x08*(0x19-0x03)) = 0x0000000000000003A30C00000000000000000000000000000000000000000000
/// From the bitcoin wiki:
/// Note that this packed format contains a sign bit in the 24th bit, and for example the negation of the above target would be 0x1b8404cb in packed format.
/// Since targets are never negative in practice, however, this means the largest legal value for the lower 24 bits is 0x7fffff.
/// Additionally, 0x008000 is the smallest legal value for the lower 24 bits since targets are always stored with the lowest possible exponent
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DifficultyBits (pub u32);

impl DifficultyBits {
    /// Convert the exponent representation into a vector of bytes that represents the actual number
    /// This can then be compared against a candidate block header hash to see if it fits the proof of work
    pub fn to_u256(&self) -> U256 {

	let exponent: u32 = ((self.0 >> 24) & 0xFF ).into(); // get the first byte. TODO: is the mask required?
	let base: u128 = (self.0 & 0xFFFFFF).into(); // next three bytes represent the base
	//vec![0, 10, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
	// convert to the special U256 type before doing the math
	
	let base = U256::from_words(0, base);

	let full_exponent: u32 = 8 * (exponent -  3);
	//println!("full_exponent = {:?}", full_exponent);
	assert!(full_exponent < 255); // we can't be too big
	// TODO: the checked_pow() meethod didn't seem to return None when there was overflow hmmm
	let rhs = U256::from_words(0, 2).pow(full_exponent);
	//println!("rhs = {:?}", rhs);	
	let difficulty_target = base * rhs;
	difficulty_target
	// U256::from_words(0x01_05_00_00_00_00_00_00_00_00_00_00_00_00_00_00, 0x00_00_00_00_00_00_00_00_00_00_00_00_00_00_00_00)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    version: u32, // 4 bytes: A version number to track software/protocol upgrades
    previous_block_hash: Hash, // 32 bytes: A reference to the hash of the previous (parent) block in the chain
    merkle_root: Hash, // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
    time_stamp: u64, // 4  (8 is ok?) bytes: The approximate creation time of this block (in seconds elapsed since Unix Epoch)
    difficulty_bits: DifficultyBits, // 4 bytes: The Proof-of-Work algorithm difficulty target for this block
    nonce: Option<u32>, // 4 bytes: A counter used for the Proof-of-Work algorithm
}


impl BlockHeader {
    pub fn new(version: u32, previous_block_hash: Hash, merkle_root: Hash, difficulty_bits: DifficultyBits) -> Self {
	Self {
	    version: version, 
	    previous_block_hash: previous_block_hash,
	    merkle_root: merkle_root, 
	    time_stamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(), // now is after unix_epoch so we can unrwap
	    difficulty_bits: difficulty_bits,
	    nonce: None, // this will get filled by the mining process
	}
    }
    
    pub fn hash(&self) -> Hash {
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
	// we use a Cursor to read a Vec<u8> into two u128s, then store them inside a U256
	let mut rdr = Cursor::new(hash_vecs);
	let hi = rdr.read_u128::<BigEndian>().unwrap();
	let low = rdr.read_u128::<BigEndian>().unwrap();
        U256::from_words(hi, low)
    }
}

#[derive(Debug)]
pub struct TransactionList {
    pub transactions: Vec<Transaction>,    
}

impl TransactionList {
    pub fn new(transactions: Vec<Transaction>) -> Self {
	Self { transactions}
    }

    pub fn push(&mut self, transaction: Transaction) {
	self.transactions.push(transaction);
    }

    // TODO: implement
    pub fn get_merkle_root(&self) -> Hash {
	U256::ZERO
    }

    pub fn len(&self) -> u32 {
	self.transactions.len() as u32
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub block_size: u32,
    pub block_header: BlockHeader,
    pub transaction_count: u32,
    pub transaction_list: TransactionList,
}

impl Block {
    /// We try multiple nonce values, each time hashing the block header wtih Sha256,
    /// once we have found a hash that satisfies the difficulty requirment,
    /// we return with self.block_header.nonce set to the appropriate value
    pub fn mine(&mut self)  {
	let difficulty_target = self.block_header.difficulty_bits.to_u256(); // what we will compare our hashes against
	let mut nonce: u32 = 0;
	loop {
	    self.block_header.nonce = Some(nonce);
	    let struct_hash = self.block_header.hash();
	    //println!("nonce = {:?}, hash = {:?}, leading_zeros = {:?}", nonce, struct_hash, struct_hash.leading_zeros());
	    //if struct_hash.is_less_than_or_equal(&difficulty_vector) {
	    if struct_hash <= difficulty_target {	    
		// we have found a difficult enough hash value, so we are done
		println!("Found a valid nonce {:?} for proof of work!", nonce);
		println!("hash as bytes = {:?}", struct_hash.to_be_bytes());		
		break;
	    }
	    nonce += 1;
	}
	//println!("Difficulty target = {:?}", difficulty_target);
	//println!("Difficulty target bytes = {:?}", difficulty_target.to_be_bytes());	
	
    }

}


    

#[cfg(test)]
mod tests {
    use super::*;
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
