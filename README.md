Work-in-progress implementation of the Bitcoin protocol. 
I am primarily using https://github.com/bitcoinbook/bitcoinbook and https://en.bitcoin.it/wiki/Main_Page as my sources. 
This project is for me to understand the inner workings of the Bitcoin protocol and to continuing developing my Rust skills.

# Currently implemented:
1. Transactions (Signing and verifying)
2. Script language (a subset of, at least)
3. Blocks and headers (and their hashing)
4. Mining/Block creation
5. Simple blockchain representation as a vec of blocks.

# Todo
1. Better chain representation (handle forks)
2. Wallets/address creation
3. Persistance/Database
4. Networking (kinda important lol)
5. Longest chain consensus
