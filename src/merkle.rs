use crate::Hash;
use crate::DoubleSHA;

use itertools::Itertools;
use sha2::{Sha256, Digest};


/// given a vec of data, construct a merkle root by repeatedly concatting pairs of hashes
/// to reduce the final result into a single hash
/// todo: need a good unit test
pub fn get_merkle_root<T: DoubleSHA>(data: &Vec<T>) -> Hash {
    let mut hashes: Vec<Hash> = data.iter().map(|d| d.sha256d()).collect(); // first hash each datum
    while hashes.len() > 1 {
        println!("len of hashes = {:?}", hashes.len());
        if hashes.len() % 2 == 1 {
            // odd number of hashes -> double the back element
            hashes.push(*hashes.last().unwrap()); // we know the vec isn't empty
            println!("double the last");
        }
        let mut new_hashes = vec![];
        for (a, b) in hashes.iter().tuples() {
            new_hashes.push(sha256d_two_hashes(a, b));
        }
        hashes = new_hashes;
    }
    hashes.pop().expect("should be exactly one last hash left")
}


/// Given two input hashes, we sha256 the concat of them twice
/// TODO: is there a better way to refactor all of this sha256 on top of U256
fn sha256d_two_hashes(a: &Hash, b: &Hash) -> Hash {
    let mut hasher_1 = Sha256::new();    
    for i in 0..4 {
        // iterate over the 4 constituent u64            
	hasher_1.update(a.0[i].to_be_bytes());
    }
    for i in 0..4 {
        // iterate over the 4 constituent u64            
	hasher_1.update(b.0[i].to_be_bytes());
    }

    let first_hash_vec: Vec<u8> = hasher_1.finalize().to_vec();    
    let mut hasher_2 = Sha256::new();
    hasher_2.update(first_hash_vec);
    let second_hash_vec = hasher_2.finalize().to_vec();
    Hash::from(&second_hash_vec[..])
}
