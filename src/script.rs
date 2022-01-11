use serde::{Serialize, Deserialize};
use bincode;

/// enum to hold the various Script operations and their associated values
/// we derive Serialize and Deserialize so that we can turn the StackOp into bytes during hashing
/// (I couldn't find a more direct way to do that like everything else, but there might be)
#[derive(Serialize, Deserialize, Debug)]
pub enum StackOp {
    PushVal(u32),
    PushKey(Box<[u8]>), // the data stored here is the byte representation of an EncodedPoint<Secp256k1>
    //PushVerifyingKey(VerifyingKey<Secp256k1>),
    //PushSigningKey(SigningKey<Secp256k1>),	
    OpAdd,
    OpSub,    
    OpDup,
    //OP_HASH_160,
    OpEqual,
    OpChecksig,
    //OP_VERIFY,
    //OP_EQ_VERIFY,
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
#[derive(Debug)]
//#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub ops: Vec<StackOp>
}


/// given an unlocking script and a locking script, this function executes them on a stack and
/// returns a bool to indicate if the unlocking script is valid for the locking script, i.e. is the
/// the associated transaction allowed
fn execute_scripts(unlocking_script: Script, locking_script: Script) -> bool {
    false
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    
    #[test]    
    fn test_valid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(5)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_simple_equal_with_extra_on_stack() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(1), StackOp::PushVal(5)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_invalid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(6)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, false);
    }

    #[test]    
    fn test_valid_add() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(3), StackOp::PushVal(2), StackOp::OpAdd]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_add_more_in_locking() {
	let locking_script = Script {ops: vec![StackOp::PushVal(2), StackOp::OpAdd, StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(3)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_valid_add_and_dup() {
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpAdd, StackOp::PushVal(8), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(4)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_sub() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(20), StackOp::PushVal(15), StackOp::OpSub]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_invalid_sub() {
	let locking_script = Script {ops: vec![StackOp::PushVal(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::PushVal(20), StackOp::PushVal(20), StackOp::OpSub]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }    

}
