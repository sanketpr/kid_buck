mod processor;
mod transaction;

use std::env;
use std::error::Error;
use std::fs::File;
use csv::Reader;
use std::collections::HashMap;
use processor::{BasicProcessor};
use transaction::{Transaction, ClientAccount};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <transactions.csv>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let file = File::open(file_path)?;
    let mut rdr = Reader::from_reader(file);

    let client_accounts = HashMap::new();
    let past_transactions = Vec::new();
    let mut processor = BasicProcessor::new(client_accounts, past_transactions);

    for result in rdr.deserialize() {
        let tx: Transaction = result?;
        let _ = processor.process_transaction(tx);
    }

    print_client_accounts_csv(&processor.client_accounts);
    Ok(())
}

pub fn print_client_accounts_csv(accounts: &HashMap<u32, ClientAccount>) {
    println!("client_id,available,held,total,locked");

    for (client_id, account) in accounts {
        println!(
            "{},{:.4},{:.4},{:.4},{}",
            client_id,
            account.available,
            account.held,
            account.total,
            account.locked
        );
    }
}

