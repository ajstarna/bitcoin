use clap::{Parser, Subcommand};
use ecdsa::{SigningKey, VerifyingKey};
use k256::{Secp256k1};
use bitcoin::blockchain::{BlockChain};

/// Way to actually spin up a blockchain
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Init and start adding to a new chain, and optional save it to the given store_path
    New {
        store_path: Option<String>
    },
    /// Load an exisiting chain from the given path and start adding to it, and optional save it to the given store_path
    From {
        from_path: String,
        store_path: Option<String>,
    },
}


fn run(from_path: Option<String>, store_path: Option<String>) {
    let mut chain = if let Some(path) = from_path {
        println!("load the existing chain from {:?}", path);
        BlockChain::new()        
    } else {
        println!("init a new chain");
        BlockChain::new()       
    };

    println!("store_path = {:?}", store_path);

    let num_blocks = 4;
    let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
    let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
    let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();    	
    for _ in 0..num_blocks {
	let mut block = chain.construct_candidate_block(public_key);
	block.mine();
	chain.add_block(block);
        chain.print_transactions();
        println!();
    }
    
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { store_path } => {

            run(None, store_path);

        }
        Commands::From {from_path, store_path } => {
            run(Some(from_path), store_path);            
        }
    }
}
