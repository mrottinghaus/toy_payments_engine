mod account_manager;
use std::env;
use crate::account_manager::AccountManager;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run -- filename.csv > output.csv");
        return;
    }
    let mut account_manager = AccountManager::default();
    // parse the csv
    let mut csv_reader = csv::Reader::from_path(args.pop().expect("No valid file path provided")).expect("CSV Reader faiuled to parse");
    for result in csv_reader.deserialize() {
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
}