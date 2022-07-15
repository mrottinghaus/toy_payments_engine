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
    fn new(client_id: u16) -> Self {
        Account {
            transactions: HashMap::new(),
            held_transactions: HashMap::new(),
            frozen: false,
            available_balance: 0.0,
            client_id,
        }
    }

    // withdrawal
    // returns true if the withdrawal was successful
    // decreasing the total and available amounts
    fn withdrawal(&mut self, amount: f64) -> bool {
        if self.available_balance >= amount {
            self.available_balance -= amount;
            true
        } else {
            false
        }
    }

    // deposit funds, increasing the total and available amounts
    fn deposit(&mut self, amount: f64) {
        self.available_balance += amount;
    }

    // the transaction goes to the held hashmap,
    // the available amount should decrease
    // the held amount should increase
    // the total should stay the same
    fn dispute(&mut self, disputed: Transaction) {
        if let Some(transaction) = self.transactions.remove(&disputed.tx) {
            self.available_balance -= transaction.amount;
            self.held_transactions.insert(transaction.tx, transaction);
        }
    }

    // the transaction goes to the held hashmap,
    // the available amount should decrease
    // the held amount should increase
    // the total should stay the same
    fn resolve(&mut self, resolved: Transaction) {
        if let Some(transaction) = self.held_transactions.remove(&resolved.tx) {
            self.available_balance += transaction.amount;
            self.transactions.insert(transaction.tx, transaction);
        }
    }

    // the transaction goes to the held hashmap,
    // the available amount should decrease
    // the held amount should increase
    // the total should stay the same
    fn chargeback(&mut self, charged_back: Transaction) {
        if let Some(_) = self.held_transactions.remove(&charged_back.tx) {
            self.frozen = true;
        }
    }

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
                self.transactions.insert(transaction.tx, transaction);
            }
            TransactionType::Withdrawal => {
                if self.withdrawal(transaction.amount) {
                    self.transactions.insert(transaction.tx, transaction);
                }
            }
            TransactionType::Dispute => {
                self.dispute(transaction);
            }
            TransactionType::Resolve => {
                self.resolve(transaction);
            }
            TransactionType::Chargeback => {
                self.chargeback(transaction);
            }
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
                let mut new_account = Account::new(transaction.client);
                // then process the tx
                new_account.process_transaction(transaction);
                // save the account
                self.accounts.insert(new_account.client_id, new_account);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::account_manager::{Account, AccountManager, Transaction, TransactionType};

    // extra function for convenience
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
            test_round(account_manager._get_account(1).get_available_amount()),
            96.0409
        );
    }

    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
    }

    #[test]
    fn test_withdrawal() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: 50.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        if account.withdrawal(trans2.amount) {
            account.transactions.insert(1, trans2);
        }
        assert_eq!(account.available_balance, 50.0);
        assert_eq!(account.get_total_amount(), 50.0);
        assert_eq!(account.get_held_amount(), 0.0);
    }

    #[test]
    fn test_failed_withdrawal() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: 150.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        if account.withdrawal(trans2.amount) {
            account.transactions.insert(1, trans2);
        }
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        assert_eq!(account.get_held_amount(), 0.0);
    }

    #[test]
    fn test_dispute() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        account.dispute(trans2);
        assert_eq!(account.available_balance, 0.0);
        assert_eq!(account.get_total_amount(), 100.0);
        assert_eq!(account.get_held_amount(), 100.0);
    }

    #[test]
    fn test_failed_dispute() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 0, // we are referring to a transaction that does not exist!
            amount: 0.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        account.dispute(trans2);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        assert_eq!(account.get_held_amount(), 0.0);
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        account.dispute(trans2);
        assert_eq!(account.get_held_amount(), 100.0);
        account.resolve(trans3);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        assert_eq!(account.get_held_amount(), 0.0);
    }

    #[test]
    fn test_failed_resolve() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 2, // we are referring to a transaction that does not exist!
            amount: 0.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        account.dispute(trans2);
        assert_eq!(account.get_held_amount(), 100.0);
        account.resolve(trans3);
        assert_eq!(account.available_balance, 0.0);
        assert_eq!(account.get_total_amount(), 100.0);
        assert_eq!(account.get_held_amount(), 100.0);
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: 100.0,
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: 0.0,
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        account.dispute(trans2);
        assert_eq!(account.available_balance, 0.0);
        assert_eq!(account.get_held_amount(), 100.0);
        // chargeback
        account.chargeback(trans3);
        assert_eq!(account.available_balance, 0.0);
        assert_eq!(account.get_total_amount(), 0.0);
        assert_eq!(account.get_held_amount(), 0.0);
    }
}
