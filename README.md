Work-in-progress implementation of the Bitcoin protocol. 
I am primarily using https://github.com/bitcoinbook/bitcoinbook and https://en.bitcoin.it/wiki/Main_Page as my sources. 
This project is for me to understand the inner workings of the Bitcoin protocol and to continuing developing my Rust skills.
Not sure how far I might get (I'm discovering how deep this goes hah)

The project is a library, and there is a very minimal in progress client CLI that can be run with:
```
    - cargo run new [save_path], which will construct a new chain from scratch and save it to the optional save_path
    - cargo run from from_path [save_path], which will load the chain from from_path and continue to add to it and save it to the optional save_path
```

# Currently implemented:
1. Transactions (Signing and verifying)
2. Script language (a subset of, at least)
3. Blocks and headers (and their hashing)
4. Merkle roots
5. Mining/Block creation
6. Simple blockchain representation as a vec of blocks.
7. Simple mempool with validity checking and priority by tip amount
8. Chain persistance on disk saved as a json representation using serde
9. Mining client

# Todo
1. Wallets/address; creation/submit transactions to mempool
2. Networking between nodes
3. Better chain representation/handle temporary forks
4. Longest chain consensus
5. light node/SPV?
6. set up some github actions
