use ecdsa::{SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use k256::{Secp256k1};
use ethnum::U256;
use elliptic_curve::sec1::{EncodedPoint};

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use crate::script::{Script, StackOp};
use crate::Hash;


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
	
	for tx_in in &self.tx_ins {
	    match tx_in {
		TxIn::TxPrevious{tx_hash, tx_out_index, unlocking_script, sequence} => {
		    let (hi, low) = tx_hash.into_words();
		    hasher.update(hi.to_be_bytes());
		    hasher.update(low.to_be_bytes());
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
	// 9867146778677399561412053178184496996625184432557161352426664471158288654564 decimal
	// 15D09B6F36496CB1D7693954A23078B60AAD40F539D3503C52A314892AB1A0E4 hex
	let hash = transaction.hash();
	assert_eq!(hash, U256::from_words(28996938242674037981331829445228525750_u128, 14191864817386241420276944889147662564_u128));
    }
}
