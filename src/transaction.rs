
pub type Hash = u32; // TODO: figure this out with a library for sha256 or what not. should be 32 bytes long

/// The unlocking script when combined with a locking script and executed on the stack satisfies
/// the requirment for ownership of the utxo
struct UnlockingScript {}

enum TXIn {
    // A transaction input can either come from a previous transaction output,
    // or if it is part of a block reward, then can be a coinbase
    TXPrevious {
	tx_hash: Hash, // Hash of the transaction that we are getting this input from
	tx_out_index: u32,// The index of the tx_out within the transaction
	unlocking_script: UnlockingScript,
	sequence: u32, // TODO: what is this haha
    },
    Coinbase {
	coinbase: u32,
	sequence: u32,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_coin_base() {
	let tx_in = TXIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	if let TXIn::Coinbase {coinbase, sequence} = tx_in {
            assert_eq!(33, coinbase);
	}
    }
}

/// the locking script formally describes the conditions needed to spend a given UTXO,
/// Usually requiring a signature from a specific address
struct LockingScript {
}

struct TXOut {
    value: u32, // number of Eves 
    locking_script: LockingScript, // AKA: ScriptPubKey, but following Master Bitcoin's convention
}

pub struct Transaction {
    version: u32,
    lock_time: u32,
    tx_ins: Vec<TXIn>,
    tx_outs: Vec<TXOut>,    
}

impl Transaction {
    pub fn is_valid_check(&self) -> bool {
	true
    }
}
