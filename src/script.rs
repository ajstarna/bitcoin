use serde::{Serialize, Deserialize};
use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};
use sha2::{Sha256, Digest};
use ecdsa::signature::{Signer, Verifier, Signature}; // trait in scope for signing a message

use elliptic_curve::sec1::{EncodedPoint};
use ethnum::U256;
use bincode;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};


use crate::Hash;

/// enum to hold the various Script operations and their associated values
/// we derive Serialize and Deserialize so that we can turn the StackOp into bytes during hashing
/// (I couldn't find a more direct way to do that like everything else, but there might be)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StackOp {
    Bool(bool),
    Val(i32),
    Bytes(Box<[u8]>), // the data stored here is the byte representation of an EncodedPoint<Secp256k1> or a hash of it
    OpAdd, // pop the top two values, and put val1 + val2 on the top of the stack
    OpSub, // pop the top two values, and put val1 (bottom) - val2 (top) on the top of the stack
    OpDup, // duplicate the top value of the stack
    OpEqual, // pop the top two values, and put val1 == val2 on the top of the stack
    OpHash160, // run the top element of the stack through hash 160    
    OpCheckSig,
    OpVerify, // mark the transaction as invalid if the top value on the stack is not true
    OpEqVerify, // combine OpEq and OpVerify in one go.
}


impl StackOp {
    pub fn to_be_bytes(&self) -> Vec<u8> {
	let encoded: Vec<u8> = bincode::serialize(self).unwrap();
	encoded
    }
}

/// The unlocking script when combined with a locking script and executed on the stack satisfies
/// the requirment for ownership of the utxo
/// the locking script formally describes the conditions needed to spend a given UTXO,
/// Usually requiring a signature from a specific address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub ops: Vec<StackOp>
}

/// TODO: add the second half of this hash (ripemd160)
pub fn hash_160_to_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()

}

/// given an unlocking script and a locking script, this function executes them on a stack and
/// returns a bool to indicate if the unlocking script is valid for the locking script, i.e. is the
/// the associated transaction allowed
/// "A transaction is valid if nothing in the combined script triggers failure and the top stack
/// item is True when the script exits."
/// The previous transaction hash (for the tx_prev that we are trying to unlock) is used for OpChecksig as
/// the "message" to verify the signature on. If OpChecksig does not occur, then this argument is not used
pub fn execute_scripts(unlocking_script: &Script, locking_script: &Script, tx_previous_hash: &[u8]) -> bool {
    let mut stack: Vec<StackOp> = Vec::new();
    for op in unlocking_script.ops.iter().chain(locking_script.ops.iter()) {
	println!("stack = {:?}", stack);	
	println!("op = {:?}", op);
	match op {
	    StackOp::Bool(val) => stack.push(StackOp::Bool(*val)),	    
	    StackOp::Val(val) => stack.push(StackOp::Val(*val)),
	    StackOp::Bytes(bytes_box) => stack.push(StackOp::Bytes(bytes_box.clone())),
	    StackOp::OpAdd => {
		// attempt to pop two numbers off the stack, add them, then put the result
		// back on the stack. If this is not possible, we return false as invalid
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val1), StackOp::Val(val2)) = (op1, op2) {
			stack.push(StackOp::Val(val1 + val2));
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    },
	    StackOp::OpSub => {
		// attempt to pop two numbers off the stack, sub them, then put the result
		// back on the stack. If this is not possible, we return false as invalid
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val2), StackOp::Val(val1)) = (op1, op2) {
			// we want to sub the bottom by the top
			stack.push(StackOp::Val(val1 - val2));
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpDup => {
		// attempt to pop a numbers off the stack, then put it back onto the stack twice
		// If this is not possible, we return false as invalid
		if let Some(op1) = stack.pop() {
		    if let StackOp::Val(val1) = op1 {
			stack.push(StackOp::Val(val1));
			stack.push(StackOp::Val(val1));			
		    } else if let StackOp::Bytes(bytes_box) = op1 {
			stack.push(StackOp::Bytes(bytes_box.clone()));
			stack.push(StackOp::Bytes(bytes_box.clone()));			
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpHash160 => {
		if let Some(op1) = stack.pop() {
		    match op1 {
			/*
			StackOp::Val(val1) => {
			    let hash = hash_160_to_bytes(&val1.to_be_bytes());
			    stack.push(StackOp::Bytes(hash.into_boxed_slice()));
			},
			 */
			StackOp::Bytes(bytes) => {
			    let hash = hash_160_to_bytes(&bytes);
			    stack.push(StackOp::Bytes(hash.into_boxed_slice()));
			}
			_ => {
			    ()
			},
		    }
		} else {
		    return false;
		}
	    }
		
	    StackOp::OpEqual => {
		// attempt to pop two numbers off the stack, and see if they are equal
		// if so, we put true on the stack
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val1), StackOp::Val(val2)) = (&op1, &op2) {
			stack.push(StackOp::Bool(val1 == val2));
		    } else if let (StackOp::Bytes(bytes1), StackOp::Bytes(bytes2)) = (&op1, &op2) {
			println!("bytes1 = {:?}", bytes1);
			println!("bytes2 = {:?}", bytes2);			
			println!("bytes1 == bytes2 = {:?}", bytes1 == bytes2);						
			stack.push(StackOp::Bool(bytes1 == bytes2));
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpCheckSig => {
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Bytes(bytes_pub), StackOp::Bytes(bytes_sig)) = (&op1, &op2) {
			let encoded_point_opt = EncodedPoint::<Secp256k1>::from_bytes(bytes_pub);
			if let Ok(encoded_point) = encoded_point_opt {
			    let public_key_opt = VerifyingKey::<Secp256k1>::from_encoded_point(&encoded_point);
			    if let Ok(public_key) = public_key_opt {
				let signature: ecdsa::Signature<Secp256k1> = Signature::from_bytes(bytes_sig).expect("problem deserializing signature");
				// the "message" that was signed was the transaction of the previous hash that
				// led to the locking script that we are currently trying to unlock.
				let verified = public_key.verify(tx_previous_hash, &signature);
				if let Ok(verified) = verified {
				    stack.push(StackOp::Bool(true));
				} else {
				    stack.push(StackOp::Bool(false));				    
				}
			} else {
			    return false;
			}
			} else {
			    // if we couldn't get a public key from the bytes
			    return false;
			}
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
		
	    }
	    StackOp::OpVerify => {
		if let Some(op1) = stack.pop() {
		    if let StackOp::Bool(val1) = op1 {
			if !val1 {
			    return false;
			}
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpEqVerify => {
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val1), StackOp::Val(val2)) = (&op1, &op2) {
			if val1 != val2 {
			    return false
			}
		    } else if let (StackOp::Bytes(bytes1), StackOp::Bytes(bytes2)) = (&op1, &op2) {
			if bytes1 != bytes2 {
			    return false
			}
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	}
    }
    println!("stack at end = {:?}", stack);
    // nothing triggered an early exit, so check if the top value is True
    if let Some(op) = stack.pop() {
	if let StackOp::Bool(val) = op {	
	    return val
	}
    }
    false
}

    

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::transaction::{Transaction, TxIn, TxOut};
    
    #[test]    
    fn test_valid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(5)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_equal_with_extra_on_stack() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(1), StackOp::Val(5)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);	
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_invalid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(6)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, false);
    }

    #[test]    
    fn test_valid_add() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(3), StackOp::Val(2), StackOp::OpAdd]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_add_more_in_locking() {
	let locking_script = Script {ops: vec![StackOp::Val(2), StackOp::OpAdd, StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(3)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_valid_add_and_dup() {
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpAdd, StackOp::Val(8), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(4)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_sub() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(20), StackOp::Val(15), StackOp::OpSub]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);	
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_invalid_sub() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(20), StackOp::Val(20), StackOp::OpSub]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);	
	assert_eq!(is_valid, false);
    }

    #[test]
    fn test_op_verify() {
	// verify should simply not return false at that moment, but the scipt ends invalid with false on top
	let locking_script = Script {ops: vec![StackOp::Bool(true), StackOp::OpVerify]};
	let unlocking_script = Script {ops: vec![StackOp::Bool(false)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, false);
    }

    fn test_op_verify2() {
	// verify will reuturn false, even though the stack would end with true on top
	let locking_script = Script {ops: vec![StackOp::Bool(false), StackOp::OpVerify]};
	let unlocking_script = Script {ops: vec![StackOp::Bool(true)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, false);
    }
    
    #[test]
    fn test_op_eq_verify() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::Val(5), StackOp::OpEqVerify]};
	let unlocking_script = Script {ops: vec![StackOp::Bool(false)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, false);
    }

    #[test]
    fn test_op_eq_verify2() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::Val(4), StackOp::OpEqVerify]};
	let unlocking_script = Script {ops: vec![StackOp::Bool(true)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, false);
    }

    #[test]    
    fn test_valid_multiple_dup() {
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpDup, StackOp::OpDup, StackOp::Val(8), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(8)]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }



    #[test]
    fn test_op_hash_160_valid() {
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let answer = hash_160_to_bytes(&b);
	let locking_script = Script {ops: vec![StackOp::OpHash160, StackOp::Bytes(answer.into_boxed_slice()), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Bytes(b.into())]};
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &[0]);
	assert_eq!(is_valid, true);
    }

    /*
    #[test]
    fn test_op_hash_160_invalid() {
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let answer = 5;
	let locking_script = Script {ops: vec![StackOp::OpHash160, StackOp::Val(answer), StackOp::OpEqVerify]};
	let unlocking_script = Script {ops: vec![StackOp::Bytes(b)]};
	let is_valid = execute_scripts(unlocking_script, locking_script, 0);
	assert_eq!(is_valid, false);
    }
     */
    
    #[test]
    fn test_signature() {
	let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	println!("priv before as bytes: {:?}", b);
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
	let b_2 = private_key.to_bytes();
	println!("priv after as bytes: {:?}", b_2);


	let msg = "message_to_sign".as_bytes();

	let sig = private_key.try_sign(msg).expect("should be able to sign here");
	// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `u8`, found struct `ecdsa::Signature`
	println!("sig: {:?}", sig);	

	let sig_as_bytes: &[u8] = sig.as_bytes();
	println!("sig as bytes: {:?}", sig_as_bytes);

	// lol there is a Signature trait and a Signature struct?
	let sig2:  ecdsa::Signature<Secp256k1> = Signature::from_bytes(sig_as_bytes).expect("problem deserializing");
	
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	let c = public_key.to_encoded_point(true).to_bytes();
	println!("public as bytes: {:?}", c);

	let verified = public_key.verify(msg, &sig);
	println!("verified = {:?}", verified);


	// sig should not work for this
	let other_msg = "not_the_message_to_sign".as_bytes();
	let verified2 = public_key.verify(other_msg, &sig);
	println!("verified2 = {:?}", verified2);



	let verified = public_key.verify(msg, &sig);
	println!("verified once more = {:?}", verified);


    }

    #[test]
    fn test_op_check_sig() {
	/*
	e.g.
	LOCKING:
	"OP_DUP OP_HASH160 7f9b1a7fb68d60c536c2fd8aeaa53a8f3cc025a8 OP_EQUALVERIFY OP_CHECKSIG"
	 */
	let priv_bytes = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
	let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&priv_bytes).unwrap();
	let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();
	let public_key_bytes = public_key.to_encoded_point(true).to_bytes();
	let pub_hash = hash_160_to_bytes(&public_key_bytes);
	
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpHash160, StackOp::Bytes(pub_hash.into_boxed_slice()), StackOp::OpEqVerify, StackOp::OpCheckSig]};

	// Note: we could probably just pass in an arbitrary hash, but lets use a "real" transaction hash
	// To keep the testing more integrated.
	
	let tx_in = TxIn::Coinbase {
	    coinbase: 33,
	    sequence: 5580,
	};
	let tx_out1 = TxOut {
	    // This is the TxOut that we would be unlocking
	    value: 222,
	    locking_script: locking_script.clone()
	};
	let transaction = Transaction {
	    version: 1,
	    lock_time: 5,
	    tx_ins: vec![tx_in],
	    tx_outs: vec![tx_out1],		
	};
	let tx_hash_bytes = transaction.hash_to_bytes();


	let sig = private_key.try_sign(&tx_hash_bytes).expect("should be able to sign the transaction hash here");
	println!("sig: {:?}", sig);	
	let sig_as_bytes = sig.as_bytes().to_vec(); //TODO: is there a way to go right from &[u8] --> Box instead of through vec?
	let unlocking_script = Script {ops: vec![StackOp::Bytes(sig_as_bytes.into_boxed_slice()), StackOp::Bytes(public_key_bytes)]};
	
	let is_valid = execute_scripts(&unlocking_script, &locking_script, &tx_hash_bytes);
	assert_eq!(is_valid, true);
	

    }

}
