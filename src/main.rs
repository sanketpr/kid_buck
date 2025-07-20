mod processor;
mod transaction;

use std::env;
use std::error::Error;
use std::fs::File;
use csv::Reader;
use std::collections::HashMap;
use processor::{BasicProcessor};
use transaction::{Transaction};

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
        println!("{:?}", &processor.process_transaction(tx));
    }

    dbg!(processor.past_transactions);
    dbg!(processor.client_accounts);

    Ok(())
}

// fn process_transaction_record(
//     tx: Transaction,
//     client_accounts: &mut HashMap<ClientId, ClientAccount>,
//     past_transactions: &mut Vec<Transaction>,
// ) -> String {
//     match tx.r#type {
//         TransactionType::Deposit => {
//             let client_account = client_accounts.get(&tx.client);
//             let mut client_account = match client_account {
//                 Some(account) => account.to_owned(),
//                 None => ClientAccount {
//                     total: 0.0,
//                     available: 0.0,
//                     held: 0.0,
//                     is_account_frozen: false
//                 }
//             };

//             client_account.total += tx.amount.expect("Transaction amount is None");
//             let client_id = tx.client;
//             past_transactions.push(tx);
//             client_accounts.insert(client_id, client_account);
//             format!("Deposit accepted for Client")
//         }
//         TransactionType::Withdrawal => {
//             let client_account = client_accounts.get(&tx.client);
//             let mut client_account = match client_account {
//                 Some(account) => account.to_owned(),
//                 None => ClientAccount {
//                     total: 0.0,
//                     available: 0.0,
//                     held: 0.0,
//                     is_account_frozen: false
//                 }
//             };

//             if client_account.total >= tx.amount.expect("Transaction amount is None") {
//                 client_account.total -= tx.amount.expect("Transaction amount is None");
//                 let client_id = tx.client;
//                 past_transactions.push(tx);
//                 client_accounts.insert(client_id, client_account);
//                 format!("Withdrawal accepted for Client #{}: New balance = ${:.2}", client_id, client_account.total)
//             } else {
//                 format!("Withdrawal rejected for Client #{}: Insufficient funds", tx.client)
//             }
//         }
//         TransactionType::Dispute => {
//             let past_transaction = past_transactions.iter().find({ |&pt|   
//                 pt.tx == tx.tx
//             });

//             let client_account = client_accounts.get(&tx.client);

//             match (past_transaction, client_account) {
//                 (Some(past_transac), Some(client_account)) => {
//                     let mut client_account = client_account.to_owned();
//                     client_account.available -= past_transac.amount.expect("Transaction amount is None");
//                     client_account.held += past_transac.amount.expect("Transaction amount is None");
//                     client_accounts.insert(tx.tx, client_account);
//                     format!("Valid transaction not found")
//                 }
//                 _ => {
//                     format!("Invalid transaction id")
//                 }
//             }
//         }
//         TransactionType::Resolve => {
//             let past_transaction = past_transactions.iter().find({ |&pt|   
//                 pt.tx == tx.tx
//             });

//             let client_account = client_accounts.get(&tx.client);

//             match (past_transaction, client_account) {
//                 (Some(past_transac), Some(client_account)) => {
//                     let mut client_account = client_account.to_owned();
//                     client_account.available += past_transac.amount.expect("Transaction amount is None");
//                     client_account.held -= past_transac.amount.expect("Transaction amount is None");
//                     client_accounts.insert(tx.tx, client_account);
//                     format!("Valid transaction not found")
//                 }
//                 _ => {
//                     format!("Invalid transaction")
//                 }
//             }
//         }
//         TransactionType::Chargeback => {
//             let past_transaction = past_transactions.iter().find({ |&pt|   
//                 pt.tx == tx.tx
//             });

//             let client_account = client_accounts.get(&tx.client);

//             match (past_transaction, client_account) {
//                 (Some(past_transac), Some(client_account)) => {
//                     let mut client_account = client_account.to_owned();
//                     client_account.total -= past_transac.amount.expect("Transaction amount is None");
//                     client_account.held -= past_transac.amount.expect("Transaction amount is None");
//                     client_account.is_account_frozen = true;
//                     client_accounts.insert(tx.tx, client_account);
//                     format!("Valid transaction not found")
//                 }
//                 _ => {
//                     format!("Invalid transaction id.")
//                 }
//             }
//         }
//     }
// }