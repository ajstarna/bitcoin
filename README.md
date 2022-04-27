Work-in-progress implementation of the Bitcoin protocol. 
I am primarily using https://github.com/bitcoinbook/bitcoinbook and https://en.bitcoin.it/wiki/Main_Page as my sources. 
This project is for me to understand the inner workings of the Bitcoin protocol and to continuing developing my Rust skills.
Not sure how far I might get (I'm discovering how deep this goes hah)

# Currently implemented:
1. Transactions (Signing and verifying)
2. Script language (a subset of, at least)
3. Blocks and headers (and their hashing)
4. Mining/Block creation
5. Simple blockchain representation as a vec of blocks.
6. simple mempool with validity checking

# Todo
1. Better chain representation
2. handle temporary forks
3. Wallets/address creation/submit transactions/Client
4. Persistance/Database
5. Networking (kinda important lol)
6. Longest chain consensus
7. UI for transactions?
