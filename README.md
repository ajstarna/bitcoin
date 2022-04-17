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
1. Better chain representation
2. handle temporary forks
3. mempool
4. Wallets/address creation/submit transactions
5. Persistance/Database
6. Networking (kinda important lol)
7. Longest chain consensus
8. UI for transactions?
