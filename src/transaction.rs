//use super::validation::{LockingScript, UnlockingScript, is_valid};

pub type Hash = u32; // TODO: figure this out with a library for sha256 or what not. should be 32 bytes long


pub enum StackOp {
    Push(u32),    
    OpAdd,
    OpDup,
    //OP_HASH_160,
    OpEqual,
    OpChecksig,
    //OP_VERIFY,
    //OP_EQ_VERIFY,
}

/// The unlocking script when combined with a locking script and executed on the stack satisfies
/// the requirment for ownership of the utxo
/// the locking script formally describes the conditions needed to spend a given UTXO,
/// Usually requiring a signature from a specific address
pub struct Script {
    pub ops: Vec<StackOp>
}

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


pub struct TxOut {
    pub value: u32, // number of Eves 
    pub locking_script: Script, // AKA: ScriptPubKey, but following Master Bitcoin's convention
}

pub struct Transaction {
    pub version: u32,
    pub lock_time: u32,
    pub tx_ins: Vec<TxIn>,
    pub tx_outs: Vec<TxOut>,    
}


pub fn is_valid(transaction: Transaction) -> bool {
    for tx_in in &transaction.tx_ins {
	if let TxIn::TxPrevious { tx_hash, tx_out_index, unlocking_script, sequence} = tx_in {
	    // we only need to consider normal tx-ins here. Coinbase transactions should be checked separately
	    println!("{}",tx_hash);
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
    fn val() {
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
