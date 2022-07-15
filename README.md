# toy_payments_engine
A toy payments engine that handles deposits, withdrawals, and deisputes for multiple accounts.
This program takes one arguement as an input, the csv file to parse, and outputs the final balance of multiple accounts.

Written by Matt Rottinghaus

# running
cargo run -- inputfile.csv > outputfile.csv

# tests
cargo test

# about
Unit tests are checking each important transaction type, and test files are available in the testfile directory.
Serde and csv should be enforcing that incoming types are valid, otherwise the file will not process.
I used testfiles/biggertestfile.csv to test all the transaction types with multiple clients.


The only running total is the available balance, all other values are calulated using a hashmap of held transactions and adding them up.
This is okay because we only calculate totals at the end so there will not be a performance detriment. This is also needed because the program 
needs to be able to reference past transactions to handle disputes.

# errors
If there is an error in parsing input, the program will not run to completion.
If there is an error with a transaction, the transaction will be ignored and not stored.

# potential concurrency
Each record is parsed independently to handle incoming transactions and not depend on a complete file. 
If it is expected that there will be concurrent streams of incoming transactions, care should be taken to ensure account changes are atomic or protected.
This can be done with an Arc or a RwLock on the Account, ensuring that only one thing can write to the account at once, but it can still be read.