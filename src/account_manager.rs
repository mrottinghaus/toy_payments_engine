use crate::account::{Account, Transaction};
use std::collections::HashMap;

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
    // output_accounts
    // outputs csv format to stdout
    pub fn output_accounts(&self) {
        println!("client, available, held, total, locked");
        // print all account info
        for client in self.accounts.values() {
            client.print();
        }
    }

    // process_transaction
    // process a single transaction, create a new account if it does not currently exist
    pub fn process_transaction(&mut self, transaction: Transaction) {
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

#[cfg(test)]
mod tests {
    use crate::account::{round, Account};
    use crate::account_manager::AccountManager;

    // extra function for convenience
    impl AccountManager {
        fn _get_account(mut self, client: u16) -> Account {
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
            round(account_manager._get_account(1).get_available_amount()),
            96.0409
        );
    }
}
