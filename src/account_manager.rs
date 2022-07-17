use crate::account::{Account, Transaction};
use std::collections::HashMap;

/// The Account Manager contains all of the accounts
pub struct AccountManager {
    accounts: HashMap<u16, Account>,
}

impl Default for AccountManager {
    fn default() -> Self {
        AccountManager {
            accounts: HashMap::new(),
        }
    }
}

impl AccountManager {
    /// outputs csv format listing each account to stdout
    pub fn output_accounts(&self) {
        println!("client, available, held, total, locked");
        // print all account info
        for client in self.accounts.values() {
            client.print();
        }
    }

    /// Get the available balance for a given client
    /// This is currently only used in the example
    pub fn _get_client_balance(&self, client_id: &u16) -> f64 {
        if let Some(client) = self.accounts.get(client_id) {
            return client.get_available_amount();
        }
        0.0
    }

    /// process a single transaction, create a new account if it does not currently exist
    /// The purpose is to find which account to apply the transaction to and then have that account run the transaction
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to be processed an affect an account's balance
    ///
    /// // Doctests are not working for binaries, but you get an example anyway.
    /// # Example
    /// ```
    /// use account_manager::AccountManager;
    /// use account::Transaction;
    ///
    /// // process a single transaction and print the result
    /// let transaction = Transaction {
    ///     r#type: TransactionType::Deposit,
    ///     client: 1,
    ///     tx: 1,
    ///     amount: Some(100.0001),
    /// };
    /// let account_manager = AccountManager::default();
    /// account_manager.process_transaction(transaction);
    /// assert_eq!(account_manager._get_client_balance(1), 1100.001);
    ///
    /// ```
    pub fn process_transaction(&mut self, transaction: Transaction) {
        // check the transaction
        if let Some(transaction) = transaction.validate() {
            // find the account
            match self.accounts.get_mut(&transaction.client) {
                Some(account) => {
                    // do not process any more transactions if the account is frozen
                    if false == account.is_frozen() {
                        account.process_transaction(transaction);
                    }
                }
                None => {
                    // Create the account:
                    let mut new_account = Account::new(transaction.client);
                    // then process the tx
                    new_account.process_transaction(transaction);
                    // save the account
                    self.accounts.insert(new_account.get_id(), new_account);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account::{round, Account};
    use crate::account_manager::AccountManager;
    use csv::{ReaderBuilder, Trim};
    use std::env;

    // extra function for convenience
    impl AccountManager {
        fn get_account(&mut self, client: u16) -> Account {
            self.accounts
                .remove(&client)
                .expect("Failed to get account!")
        }
    }

    #[test]
    fn test_basic_file_balances() {
        let mut account_manager = AccountManager::default();
        // parse the csv
        let mut csv_reader = csv::Reader::from_path("testfiles/testsingleclient.csv")
            .expect("Failed to read input file testfiles/testfile.csv");
        for result in csv_reader.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            match result {
                Ok(transaction) => {
                    println!("{:?}", transaction);
                    account_manager.process_transaction(transaction);
                }
                Err(error) => {
                    println!("Failed to deserialize a transaction: {:?}", error);
                    return;
                }
            }
        }
        assert_eq!(
            round(account_manager.get_account(1).get_available_amount()),
            96.0409
        );
    }

    #[test]
    fn test_single_client() {
        let mut args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            println!("Usage: cargo run -- filename.csv > output.csv");
            return;
        }
        let mut account_manager = AccountManager::default();
        // parse the csv
        let mut csv_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(args.pop().expect("No valid file path provided"))
            .expect("CSV Reader faiuled to parse");
        for result in csv_reader.deserialize() {
            match result {
                Ok(transaction) => {
                    account_manager.process_transaction(transaction);
                }
                Err(error) => {
                    println!("Failed to deserialize a transaction: {:?}", error);
                    return;
                }
            }
        }
        assert_eq!(account_manager._get_client_balance(&1), 201.0);
        assert_eq!(account_manager.get_account(1).get_held_amount(), 1000.0);
        assert_eq!(account_manager.get_account(1).is_frozen(), true);
    }
}
