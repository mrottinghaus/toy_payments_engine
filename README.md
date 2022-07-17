# toy_payments_engine
A toy payments engine that handles deposits, withdrawals, and deisputes for multiple accounts.
This program takes one arguement as an input, the csv file to parse, and outputs the final balance of multiple accounts.

Written by Matt Rottinghaus

# running
cargo run -- transactions.csv > accounts.csv

# tests
cargo test

Please explore using the test input files in testfiles directory.

# about
This program takes in a csv file with the following fields:
 * type: the transaction type.
 * client: a unique u16 value to identify the client's account.
 * tx: a unique u32 transaction identifier.
 * amount: the amount for a deposit or withdrawal.


This program will output all of the client's accounts in csv format with the following fields:

 * client: identifier for the client
 * available: the amount available to be used by the client
 * held: the amount disputed/held
 * total: the total sum of the available balance and held funds
 * locked: true if the account is frozen


Unit tests are checking each important transaction type, and test files are available in the testfile directory.
Serde and csv should be enforcing that incoming types are valid, otherwise the file will not process.
I used testfiles/biggertestfile.csv to test all the transaction types with multiple clients.


The only running total is the available balance, all other values are calulated using a hashmap of held transactions and adding them up.
This is okay because we only calculate totals at the end so there will not be a performance detriment. This is also needed because the program 
needs to be able to reference past transactions to handle disputes.


This program expects proper csv formats, but this should go without saying.
For example, for transactions that do not have an amount, there must still be a trailing comma to denote the field.

Correct:

dispute,    1, 1,

Incorrect:

dispute,    1, 1

## transaction types
There are 5 possible transaction types:

 * Deposit - increase a client's available balance.
 * Withdrawal - decrease a client's available balance.
 * Dispute - dispute a Deposit, referenced by a transaction id, the amount will be held.
 * Resolve - resolve a disputed Deposit, the amount will be removed from the held amount and added to the available balance.
 * Chargeback - reverse a deposit, lowering the held amount. This will freeze the client's account.

## transaction validation
Transactions must be valid to be processed.
It must first be valid csv and have the parsable fields specified.
For deposits and withdrawals, there must be some positive value. Deposits and Withdrawals with no amount, negative amounts, or abnormal float values will not be processed.

The amount is not checked for Disputes, Resolves, and Chargebacks since they reference an amount in previous Deposit.

## freezing
An account will be frozen if a changeback occurs, this means no future transaction will be applied to the account.
This is not reversible in this implementation.

## errors
If there is an error in parsing input, the program will not run to completion.

If there is an error with a transaction, the transaction will be ignored and not stored.

### potential concurrency
Each record is parsed independently to handle incoming transactions and not depend on a complete file. 
If it is expected that there will be concurrent streams of incoming transactions, care should be taken to ensure account changes are atomic or protected.
This can be done with an Arc or a RwLock on the Account, ensuring that only one thing can write to the account at once, but it can still be read.
The account type should be written as to not care if it is accessed via an async task or thread. What matters is that only one thing can write an an account at a time.
