type Hash = u32; // TODO: figure this out with a library for sha256 or what not. should be 32 bytes long

struct Transaction {
}

struct BlockHeader {
    version: str, // 4 bytes: A version number to track software/protocol upgrades
    previous_block_hash: Hash, // 32 bytes: A reference to the hash of the previous (parent) block in the chain
    merkle_root: Hash, // 32 bytes: A hash of the root of the merkle tree of this block’s transactions
    time_stamp: u32, // 4 bytes: The approximate creation time of this block (in seconds elapsed since Unix Epoch)
    difficulty_target: u32, // 4 bytes: The Proof-of-Work algorithm difficulty target for this block
    nonce: u32, // 4 bytes: A counter used for the Proof-of-Work algorithm
}

struct Block {
    block_size: u32,
    block_header: BlockHeader,
    transaction_count: u32,
    transactions: Vec<Transaction>,
    
}

fn main() {
    println!("Hello, world!");
}

