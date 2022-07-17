use crate::account_manager::AccountManager;
use csv::{ReaderBuilder, Trim};
use std::env;

mod account;
mod account_manager;

fn main() {
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
    account_manager.output_accounts();
}
