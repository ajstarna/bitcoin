use serde::{Serialize, Deserialize};
use bincode;

/// enum to hold the various Script operations and their associated values
/// we derive Serialize and Deserialize so that we can turn the StackOp into bytes during hashing
/// (I couldn't find a more direct way to do that like everything else, but there might be)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StackOp {
    Bool(bool),
    Val(i32),
    Key(Box<[u8]>), // the data stored here is the byte representation of an EncodedPoint<Secp256k1>
    //PushVerifyingKey(VerifyingKey<Secp256k1>),
    //PushSigningKey(SigningKey<Secp256k1>),	
    OpAdd, // pop the top two values, and put val1 + val2 on the top of the stack
    OpSub, // pop the top two values, and put val1 (bottom) - val2 (top) on the top of the stack
    OpDup, // duplicate the top value of the stack
    //OP_HASH_160,
    OpEqual, // pop the top two values, and put val1 == val2 on the top of the stack
    OpChecksig,
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
#[derive(Debug)]
//#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub ops: Vec<StackOp>
}


/// given an unlocking script and a locking script, this function executes them on a stack and
/// returns a bool to indicate if the unlocking script is valid for the locking script, i.e. is the
/// the associated transaction allowed
/// "A transaction is valid if nothing in the combined script triggers failure and the top stack
/// item is True when the script exits."
fn execute_scripts(unlocking_script: Script, locking_script: Script) -> bool {
    let mut stack: Vec<StackOp> = Vec::new();
    for op in unlocking_script.ops.iter().chain(locking_script.ops.iter()) {
	println!("{:?}", op);
	match op {
	    StackOp::Bool(val) => stack.push(StackOp::Bool(*val)),	    
	    StackOp::Val(val) => stack.push(StackOp::Val(*val)),
	    StackOp::Key(bytes_box) => stack.push(StackOp::Key(bytes_box.clone())),
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
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    //StackOp::OP_HASH_160 => (),
	    StackOp::OpEqual => {
		// attempt to pop two numbers off the stack, and see if they are equal
		// if so, we put true on the stack
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val1), StackOp::Val(val2)) = (op1, op2) {
			stack.push(StackOp::Bool(val1 == val2));
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpChecksig => (),
	    StackOp::OpVerify => {
		if let Some(op1) = stack.pop() {
		    if let StackOp::Bool(val1) = op1 {
			return val1;
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	    StackOp::OpEqVerify => {
		if let (Some(op1), Some(op2)) = (stack.pop(), stack.pop()) {
		    if let (StackOp::Val(val1), StackOp::Val(val2)) = (op1, op2) {
			return val1 == val2;
		    } else {
			return false;
		    }
		} else {
		    return false;
		}
	    }
	}
    }
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
    
    #[test]    
    fn test_valid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(5)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_equal_with_extra_on_stack() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(1), StackOp::Val(5)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_invalid_simple_equal() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(6)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, false);
    }

    #[test]    
    fn test_valid_add() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(3), StackOp::Val(2), StackOp::OpAdd]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_add_more_in_locking() {
	let locking_script = Script {ops: vec![StackOp::Val(2), StackOp::OpAdd, StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(3)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }
    
    #[test]    
    fn test_valid_add_and_dup() {
	let locking_script = Script {ops: vec![StackOp::OpDup, StackOp::OpAdd, StackOp::Val(8), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(4)]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_valid_sub() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(20), StackOp::Val(15), StackOp::OpSub]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, true);
    }

    #[test]    
    fn test_invalid_sub() {
	let locking_script = Script {ops: vec![StackOp::Val(5), StackOp::OpEqual]};
	let unlocking_script = Script {ops: vec![StackOp::Val(20), StackOp::Val(20), StackOp::OpSub]};
	let is_valid = execute_scripts(unlocking_script, locking_script);
	assert_eq!(is_valid, false);
    }    

}
