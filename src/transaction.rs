use serde::{Serialize, Deserialize};
//use ecdsa::{SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
//use k256::{Secp256k1};

use crate::script::{Script};
use crate::{Hash};
use crate::DoubleSHA;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxIn {
    // A transaction input can either come from a previous transaction output,
    // or if it is part of a block reward, then can be a coinbase
    TxPrevious {
	tx_hash: Hash, // Hash of the transaction that we are getting this input from
	tx_out_index: usize,// The index of the tx_out within the transaction
	unlocking_script: Script, // AKA: ScriptSig, but lets follow Mastering Bitcoin's convention
	sequence: u32, // TODO: what is this haha
    },
    Coinbase {
	coinbase: u32,
	sequence: u32,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u32, // number of satoshis 
    pub locking_script: Script, // AKA: ScriptPubKey, but following Master Bitcoin's convention
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub lock_time: u32,
    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,    
}

impl Transaction {
    pub fn hash_to_bytes(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.lock_time.to_be_bytes());
	
	for tx_in in &self.tx_ins {
	    match tx_in {
		TxIn::TxPrevious{tx_hash, tx_out_index, unlocking_script, sequence} => {
                    for i in 0..4 {
		        hasher.update(tx_hash.0[i].to_be_bytes());
                    }
		    hasher.update(tx_out_index.to_be_bytes());
		    for op in &unlocking_script.ops {
			hasher.update(op.to_be_bytes());
		    }
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
	    for op in &tx_out.locking_script.ops {
		hasher.update(op.to_be_bytes());
	    } 
	}
	hasher.finalize().to_vec()
    }
	
    /// hash all the bytes of the transaction
    /// TODO: is there a "nicer" way to do this rather than like depth first iterating through the whole data structure?
    /// TODO: could we use serde to turn into bytes then simply hash that? is serde deterministic?x	
    pub fn hash(&self) -> Hash {
	//let hash_vecs: Vec<u8> = hasher.finalize().to_vec();
	let hash_vec: Vec<u8> = self.hash_to_bytes();
        Hash::from(&hash_vec[..])
    }

}

impl DoubleSHA for Transaction {

    fn sha256d(&self) -> Hash {
	let first_hash_vec: Vec<u8> = self.hash_to_bytes();
        let mut hasher_2 = Sha256::new();
        hasher_2.update(first_hash_vec);
        let second_hash_vec = hasher_2.finalize().to_vec();
        Hash::from(&second_hash_vec[..])
    }
}

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    InvalidScript,    
    OverSpend,
    CoinbaseSpend,
    TxInNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::StackOp;
    
    #[test]
    fn test_coin_base() {
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	if let TxIn::Coinbase {coinbase, sequence: _} = tx_in {
            assert_eq!(33, coinbase);
	}
    }

    #[test]
    fn test_hash_transaction() {
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	let tx_out1 = TxOut {
	    value: 222,
	    locking_script: Script {ops: vec![StackOp::OpDup]},	    
	};
	let tx_out2 = TxOut {
	    value: 333,
	    locking_script: Script {ops: vec![StackOp::OpEqual]},	    	    
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out1, tx_out2],		
	};

	// note: this is simply the hash that comes out when i presently run it.
	// This will at least show if something changes unexpectedly in the future
	// 34955240534511464281001754460162170822162357866488741313605570467957795050853 decimal
	// 4D47F70BE4C85658925EC886B1C362EBDBC96D9E41F1DC55520169815A11D565 hex
        // Note: this has changed multiple times as i impliment, so is it even a good test..?
	let hash = transaction.hash();
        println!("hash = {:?}", hash);
        let answer = Hash::from([0x4D, 0x47, 0xF7, 0x0B, 0xE4, 0xC8, 0x56, 0x58, 0x92, 0x5E, 0xC8, 0x86, 0xB1, 0xC3, 0x62, 0xEB, 0xDB,
                                 0xC9, 0x6D, 0x9E, 0x41, 0xF1, 0xDC, 0x55, 0x52, 0x01, 0x69, 0x81, 0x5A, 0x11, 0xD5, 0x65]);
	assert_eq!(hash, answer);
    }
}
