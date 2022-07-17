use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Round an f64 to 4 decimal places of precision.
pub fn round(num: f64) -> f64 {
    let temp = (num * 10000.0) as i32;
    return temp as f64 / 10000.0;
}

/// The possible kinds of transactions that can be processed
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Contains all information relevant to a single transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl Transaction {
    /// Validate a transaction
    /// A transaction amount for a Withdrawal or Deposit is only valid
    /// if the amount is Some and positive but finite.
    /// This returns None if the transaction should be ignored and discarded
    ///
    /// # Note
    /// if not doing a move is important for performance or memory usage,
    /// this method can be changed to take &self and return a bool.
    /// It is implemented this way to prevent using the transaction after it has been invalidated.
    pub fn validate(self) -> Option<Self> {
        // Amounts only apply to withdrawals and deposits
        if (self.r#type == TransactionType::Withdrawal) || (self.r#type == TransactionType::Deposit)
        {
            match self.amount {
                // The amount must not be None
                Some(amount) => {
                    // It must be some positive value
                    if amount.is_normal() && amount.is_sign_positive() {
                        Some(self)
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            Some(self)
        }
    }
}

/// Represents a single client's account information
/// This should only contain transactions that apply to one client
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
    /// Returns a new client account
    ///
    /// # Arguments
    ///
    /// * `client_id` - a unique u16 that identifies this client
    pub fn new(client_id: u16) -> Self {
        Account {
            transactions: HashMap::new(),
            held_transactions: HashMap::new(),
            frozen: false,
            available_balance: 0.0,
            client_id,
        }
    }

    /// Handle a withdrawal transaction type
    /// returns true if the withdrawal was successful
    /// decreasing the total and available amounts
    ///
    /// # Arguments
    ///
    /// * `amount` - a positive f64 of the amount to be subracted from the balance
    fn withdrawal(&mut self, amount: f64) -> bool {
        if self.available_balance >= amount {
            self.available_balance -= amount;
            true
        } else {
            false
        }
    }

    /// deposit funds, increasing the total and available amounts
    /// # Arguments
    ///
    /// * `amount` - a positive f64 of the amount to be added to the balance
    fn deposit(&mut self, amount: f64) {
        self.available_balance += amount;
    }

    /// the transaction goes to the held hashmap,
    /// the available amount should decrease
    /// the held amount should increase
    /// the total should stay the same
    /// # Arguments
    ///
    /// * `disputed` - the Disputed type Transaction to be processed
    fn dispute(&mut self, disputed: Transaction) {
        if let Some(transaction) = self.transactions.remove(&disputed.tx) {
            self.available_balance -= transaction.amount.unwrap_or(0.0);
            self.held_transactions.insert(transaction.tx, transaction);
        }
    }

    /// the transaction goes to the held hashmap,
    /// the available amount should decrease
    /// the held amount should increase
    /// the total should stay the same
    /// # Arguments
    ///
    /// * `resolved` - the Resolve type Transaction to be processed
    fn resolve(&mut self, resolved: Transaction) {
        if let Some(transaction) = self.held_transactions.remove(&resolved.tx) {
            self.available_balance += transaction.amount.unwrap_or(0.0);
            self.transactions.insert(transaction.tx, transaction);
        }
    }

    /// the transaction goes to the held hashmap,
    /// the available amount decreases
    /// the held amount increases
    /// the total remains unchanged
    /// # Arguments
    ///
    /// * `charged_back` - the Chargeback type Transaction to be processed
    fn chargeback(&mut self, charged_back: Transaction) {
        if let Some(_) = self.held_transactions.remove(&charged_back.tx) {
            self.frozen = true;
        }
    }

    /// Return the amount available to the client
    pub fn get_available_amount(&self) -> f64 {
        self.available_balance
    }

    /// Return the held amount - the total balance in dispute
    pub fn get_held_amount(&self) -> f64 {
        let mut total = 0.0;
        for value in self.held_transactions.values() {
            total += value.amount.unwrap_or(0.0);
        }
        total
    }

    /// Return the sum of the available balance and the funds held in dispute
    pub fn get_total_amount(&self) -> f64 {
        self.available_balance + self.get_held_amount()
    }

    /// Returns true if the client's account is frozen and should not process transactions
    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Get the unique u16 identifier of the client
    pub fn get_id(&self) -> u16 {
        self.client_id
    }

    /// Process a single transaction that applies to this account
    /// This is the main functionality of an account
    /// # Arguments
    ///
    /// * `charged_back` - the Chargeback type Transaction to be processed
    pub fn process_transaction(&mut self, transaction: Transaction) {
        match transaction.r#type {
            TransactionType::Deposit => {
                self.deposit(transaction.amount.unwrap_or(0.0));
                self.transactions.insert(transaction.tx, transaction);
            }
            TransactionType::Withdrawal => {
                if self.withdrawal(transaction.amount.unwrap_or(0.0)) {
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

    /// output the required csv fields for this account
    /// Returns the following fields: client, available, held, total, locked
    pub fn print(&self) {
        println!(
            "{:?}, {:?}, {:?}, {:?}, {:?}",
            self.client_id,
            round(self.get_available_amount()),
            round(self.get_held_amount()),
            round(self.get_total_amount()),
            self.frozen
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::account::{Account, Transaction, TransactionType};

    // Test Transaction validation
    #[test]
    fn test_valid_transactions() {
        let transaction = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(44.99),
        };
        assert!(transaction.validate().is_some());
        let transaction = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: Some(44.99),
        };
        assert!(transaction.validate().is_some());
        let transaction = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        assert!(transaction.validate().is_some());
        let transaction = Transaction {
            r#type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        };
        assert!(transaction.validate().is_some());
        let transaction = Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        };
        assert!(transaction.validate().is_some());
    }

    #[test]
    fn test_invalid_transaction_amounts() {
        let transaction = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(-44.99),
        };
        assert!(transaction.validate().is_none());
        let transaction = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: Some(-44.99),
        };
        assert!(transaction.validate().is_none());
        let transaction = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(0.0),
        };
        assert!(transaction.validate().is_none());
        let transaction = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: Some(f64::INFINITY),
        };
        assert!(transaction.validate().is_none());
        let transaction = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,
        };
        assert!(transaction.validate().is_none());
    }

    // Test Account
    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        let trans1 = Transaction {
            r#type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(100.0),
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(50.0),
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        if account.withdrawal(trans2.amount.unwrap_or(0.0)) {
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(150.0),
        };
        account.transactions.insert(1, trans1);
        account.deposit(100.0);
        assert_eq!(account.available_balance, 100.0);
        assert_eq!(account.get_total_amount(), 100.0);
        if account.withdrawal(trans2.amount.unwrap_or(0.0)) {
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: Some(0.0),
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 0, // we are referring to a transaction that does not exist!
            amount: None,
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 2, // we are referring to a transaction that does not exist!
            amount: None,
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
            amount: Some(100.0),
        };
        let trans2 = Transaction {
            r#type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        };
        let trans3 = Transaction {
            r#type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
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
        assert_eq!(account.frozen, true);
    }
}
