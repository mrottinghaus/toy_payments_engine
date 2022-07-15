use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: f64,
}

pub struct Account {
    transactions: HashMap<u32, Transaction>,
    held_transactions: HashMap<u32, Transaction>,
    available_balance: f64,
    frozen: bool,
    client_id: u16,
}

impl Default for Account {
    fn default() -> Self {
        Account {
            transactions: HashMap::new(),
            held_transactions: HashMap::new(),
            frozen: false,
            available_balance: 0.0,
            client_id: 0,
        }
    }
}

impl Account {
    fn withdrawal(&mut self, amount: f64) {
        if self.available_balance >= amount {
            self.available_balance -= amount;
        }
    }

    fn deposit(&mut self, amount: f64) {
        self.available_balance += amount;
    }

    fn dispute(&mut self) {}

    fn resolve(&mut self) {}

    fn chargeback(&mut self) {}

    fn get_available_amount(&self) -> f64 {
        self.available_balance
    }

    fn get_held_amount(&self) -> f64 {
        let mut total = 0.0;
        for value in self.held_transactions.values() {
            total += value.amount;
        }
        total
    }

    fn get_total_amount(&self) -> f64 {
        self.available_balance + self.get_held_amount()
    }

    // process a single transaction that applies to this account
    pub fn process_transaction(&mut self, transaction: Transaction) {
        match transaction.r#type {
            TransactionType::Deposit => {
                self.deposit(transaction.amount);
            }
            TransactionType::Withdrawal => {
                self.withdrawal(transaction.amount);
            }
            TransactionType::Dispute => {}
            TransactionType::Resolve => {}
            TransactionType::Chargeback => {}
        }
    }

    // output the required csv fields for this account
    pub fn print(&self) {
        // client, available, held, total, locked
        println!(
            "{:?}, {:?}, {:?}, {:?}, {:?}",
            self.client_id,
            self.get_available_amount(),
            self.get_held_amount(),
            self.get_total_amount(),
            self.frozen
        );
    }
}

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
    fn output_accounts(&self) {
        println!("client, available, held, total, locked");
        // TODO print all account info
    }

    // process_transaction
    // process a single transaction, create a new account if it does not currently exist
    pub fn process_transaction(&mut self, transaction: Transaction) {
        // find the account
        match self.accounts.get_mut(&transaction.client) {
            Some(account) => {
                account.process_transaction(transaction);
            }
            None => {
                // Create the account:
                let mut new_account = Account::default();
                let account_id = transaction.client;
                // then process the tx
                new_account.process_transaction(transaction);
                // save the account
                self.accounts.insert(account_id, new_account);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account_manager::{Account, AccountManager};

    // extra function to expose the account for testing
    impl AccountManager {
        fn _get_account(mut self, client: u16) -> Account {
            self.accounts
                .remove(&client)
                .expect("Failed to get account!")
        }
    }

    fn test_round(num: f64) -> f64 {
        let temp = (num * 10000.0) as i32;
        return temp as f64 / 10000.0;
    }

    #[test]
    fn test_basic_file_balances() {
        let mut account_manager = AccountManager::default();
        // parse the csv
        let mut csv_reader = csv::Reader::from_path("testsingleclient.csv")
            .expect("Failed to read input file testfile.csv");
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
            test_round(account_manager._get_account(1).get_available_amount()),
            96.0409
        );
    }
}
