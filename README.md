# adam_coin
Work-in-progress implementation of the Bitcoin protocol. 
I am primarily using https://github.com/bitcoinbook/bitcoinbook and https://en.bitcoin.it/wiki/Main_Page as my sources. 
This project is for me to understand the inner workings of the Bitcoin protocol and to continuing developing my Rust skills.

# Currently implemented:
Transactions (Signing and verifying)
Script language (a subset of, at least)
Blocks and headers (and their hashing)
Mining/Block creation
Simple blockchain representation as a vec of blocks.

# Todo
Better chain representation (handle forks)
Wallets/address creation
Persistance/Database
Networking (kinda important lol)
Longest chain consensus
