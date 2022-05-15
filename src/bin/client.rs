use clap::{Parser, Subcommand};
use ecdsa::{SigningKey, VerifyingKey};
use std::sync::atomic::{AtomicBool, Ordering};
use ctrlc;
use std::sync::Arc;
use std::error;
use std::fs::File;
use std::io::{BufReader, Write};

//use serde::{Serialize, Deserialize};

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
    /// Init and start adding to a new chain, and optional save it to the given save_path
    New {
        save_path: Option<String>
    },
    /// Load an exisiting chain from the given path and start adding to it, and optional save it to the given save_path
    From {
        from_path: String,
        save_path: Option<String>,
    },
}


fn run(from_path: Option<String>, save_path: Option<String>) -> Result<(), Box<dyn error::Error>>{
    let mut chain = if let Some(path) = from_path {
        println!("load the existing chain from {:?}", path);
        // TODO from_path method
        let file = File::open(path)?;
        println!("file = {:?}", file);
        let reader = BufReader::new(file);
        // Read the JSON contents of the file as an Blockchain`.
        serde_json::from_reader(reader)?
    } else {
        println!("init a new chain");
        BlockChain::new()       
    };

    println!("save_path = {:?}", save_path);
    let b = "adamadamadamadamadamadamadamadam".as_bytes(); // arbitrary for testing. 32 long
    let private_key: SigningKey<Secp256k1> = SigningKey::<Secp256k1>::from_bytes(&b).unwrap();
    let public_key: VerifyingKey<Secp256k1> = private_key.verifying_key();

    let running = Arc::new(AtomicBool::new(true)); // this bool tells the process to keep looping
    let r = running.clone(); // need a clone that we can pass into the ctrl closure handling
    ctrlc::set_handler(move || {
        println!("\n\nctrl-c was detected!");
        r.store(false, Ordering::SeqCst); // this tells the loop that we are done adding blocks
    }).expect("Error setting Ctrl-C handler");
    
    while running.load(Ordering::SeqCst) {        
	let mut block = chain.construct_candidate_block(public_key);
	block.mine();
	chain.add_block(block);
        //chain.print_transactions();
        println!();
    }

    if let Some(path) = save_path {
        println!("saving chain to {:?}", path);
        let serialized = serde_json::to_string(&chain).unwrap();
        //println!("chain:");
        //println!("{:?}", chain);
        //chain.print_transactions();
        //println!("serialized = {:?}", serialized);
        let mut file = File::create(path)?;
        write!(file, "{}", serialized)?;
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::New { save_path } => {

            run(None, save_path)

        }
        Commands::From {from_path, save_path } => {
            run(Some(from_path), save_path)
        }
    };
    println!("Result = {:?}", result);
}
