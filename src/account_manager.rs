use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType
{
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction
{
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: f64,
}

pub struct Account
{
    available_balance: f64,
    frozen: bool,
}

impl Default for Account {
    fn default() -> Self {
        Account {
            available_balance: 0.0,
            frozen: false,
        }
    }
}

impl Account {}

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

    pub fn process_transaction(&mut self, transaction: Transaction) {
        //
    }

}