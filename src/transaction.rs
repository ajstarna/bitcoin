use ecdsa::{SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use k256::{Secp256k1};
use ethnum::U256;
use elliptic_curve::sec1::{EncodedPoint};

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};
use bincode;

pub type Hash = U256;

//#[derive(Serialize, Deserialize, Debug)]
#[derive(Debug)]
pub enum StackOp {
    PushVal(u32),
    PushKey(EncodedPoint<Secp256k1>),
    //PushVerifyingKey(VerifyingKey<Secp256k1>),
    //PushSigningKey(SigningKey<Secp256k1>),	
    //OpAdd,
    OpDup,
    //OP_HASH_160,
    OpEqual,
    OpChecksig,
    //OP_VERIFY,
    //OP_EQ_VERIFY,
}

impl StackOp {
    fn to_be_bytes(&self) -> Vec<u8> {
	//let encoded: Vec<u8> = bincode::serialize(self).unwrap();
	//encoded
	match_
    }
}

/// The unlocking script when combined with a locking script and executed on the stack satisfies
/// the requirment for ownership of the utxo
/// the locking script formally describes the conditions needed to spend a given UTXO,
/// Usually requiring a signature from a specific address
#[derive(Debug)]
pub struct Script {
    pub ops: Vec<StackOp>
}

#[derive(Debug)]
pub enum TxIn {
    // A transaction input can either come from a previous transaction output,
    // or if it is part of a block reward, then can be a coinbase
    TxPrevious {
	tx_hash: Hash, // Hash of the transaction that we are getting this input from
	tx_out_index: u32,// The index of the tx_out within the transaction
	unlocking_script: Script, // AKA: ScriptSig, but lets follow Mastering Bitcoin's convention
	sequence: u32, // TODO: what is this haha
    },
    Coinbase {
	coinbase: u32,
	sequence: u32,
    }
}

#[derive(Debug)]
pub struct TxOut {
    pub value: u32, // number of Eves 
    pub locking_script: Script, // AKA: ScriptPubKey, but following Master Bitcoin's convention
}

#[derive(Debug)]
pub struct Transaction {
    pub version: u32,
    pub lock_time: u32,
    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,    
}

impl Transaction {

    /// hash all the bytes of the transaction
    /// TODO: is there a "nicer" way to do this rather than like depth first iterating through the whole data structure?
    /// TODO: could we use serde to turn into bytes then simply hash that? is serde deterministic?
    fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.lock_time.to_be_bytes());


	// todo: can we just use serde or whatever on the structs then hash that??
	
	for tx_in in &self.tx_ins {
	    match tx_in {
		TxIn::TxPrevious{tx_hash, tx_out_index, unlocking_script, sequence} => {
		    let (hi, low) = tx_hash.into_words();
		    hasher.update(hi.to_be_bytes());
		    hasher.update(low.to_be_bytes());
		    hasher.update(tx_out_index.to_be_bytes());
		    /*
		    for op in &unlocking_script.ops {
			hasher.update(op.to_be_bytes());
		    } */
		    hasher.update(sequence.to_be_bytes());		    		    
		},
		TxIn::Coinbase{coinbase, sequence} => {
		    hasher.update(coinbase.to_be_bytes());
		    hasher.update(sequence.to_be_bytes());
		}
	    }
	}
	for tx_out in &self.tx_outs {
	    hasher.update(tx_out.value.to_be_bytes());
	    /*
	    for op in &tx_out.locking_script.ops {
		hasher.update(op.to_be_bytes());
	    } */
	}
	let hash_vecs: Vec<u8> = hasher.finalize().to_vec();
	// we use a Cursor to read a Vec<u8> into two u128s, then store them inside a U256
	let mut rdr = Cursor::new(hash_vecs);
	let hi = rdr.read_u128::<BigEndian>().unwrap();
	let low = rdr.read_u128::<BigEndian>().unwrap();
        U256::from_words(hi, low)
    }

}

pub fn is_valid(transaction: Transaction) -> bool {
    for tx_in in &transaction.tx_ins {
	if let TxIn::TxPrevious { tx_hash, tx_out_index, unlocking_script, sequence} = tx_in {
	    // we only need to consider normal tx-ins here. Coinbase transactions should be checked separately
	    println!("{:?}", tx_hash);
	}

    }
    true
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_coin_base() {
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	if let TxIn::Coinbase {coinbase, sequence} = tx_in {
            assert_eq!(33, coinbase);
	}
    }

    #[test]
    fn test_invalid_block() {
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	let tx_out = TxOut { value: 2, locking_script: Script {ops: vec![StackOp::OpDup]} };
	let transaction = Transaction {
	    version: 1,
	    lock_time: 100,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out],	    
	};

	assert_eq!(is_valid(transaction), false);
    }
}
