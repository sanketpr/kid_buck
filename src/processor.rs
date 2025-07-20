use crate::transaction::{ClientAccount, ClientId, Transaction, TransactionType};
use std::collections::HashMap;

pub struct BasicProcessor {
    pub client_accounts: HashMap<ClientId, ClientAccount>,
    pub past_transactions: Vec<Transaction>,
}

impl BasicProcessor {
    pub fn new(client_account: HashMap<ClientId, ClientAccount>, past_transactions: Vec<Transaction>) -> Self {
        BasicProcessor {
            client_accounts: client_account,
            past_transactions: past_transactions
        }
    }
}

impl BasicProcessor {
    pub fn process_transaction(
        &mut self,
        tx: Transaction
    ) -> String {
        let client_id = &tx.client;

        if !self.is_valid_transaction(&tx) {
            return "Invalid transaction".to_owned();
        }

        match tx.r#type {
            TransactionType::Deposit => {
                let mut client_account = match self.client_accounts.get(client_id) {
                    Some(account) => account.to_owned(),
                    None => ClientAccount::new()
                };
                let res = self.process_deposit(&tx, &mut client_account);
                match res {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account);
                        self.past_transactions.push(tx);
                        format!("Deposit. Client id {}, Transaction id {}, Amount {}", tx.client, tx.tx, tx.amount.unwrap())
                    },
                    Err(e) => {
                        e
                    }
                }
            }
            TransactionType::Withdrawal => {
                let client_account = self.client_accounts.get(client_id).cloned();
                let res = client_account.ok_or("Account not found".to_owned())
                    .and_then(|account| self.process_withdrawal(&tx, &account));

                match res {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account.expect("expected client account"));
                        self.past_transactions.push(tx);
                        format!("Withdraw. Client id {}, Transaction id {}, Amount {}", tx.client, tx.tx, tx.amount.unwrap())
                    },
                    Err(e) => e
                }
            }
            TransactionType::Dispute => {
                let client_account = self.client_accounts.get(client_id).cloned();
                let res = client_account.ok_or("Account not found".to_owned())
                    .and_then(|mut account| self.process_dispute(&tx, &mut account));

                match res {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account.expect("expected client account"));
                        self.past_transactions.push(tx);
                        format!("Disputed. Client id {}, Transaction id {}", tx.client, tx.tx)
                    },
                    Err(e) => e
                }
            }
            TransactionType::Resolve => {
                let client_account = self.client_accounts.get(client_id).cloned();
                let res = client_account.ok_or("Account not found".to_owned())
                    .and_then(|mut account| self.process_resolve(&tx, &mut account));

                match res {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account.expect("expected client account"));
                        self.past_transactions.push(tx);
                        format!("Resolved. Client id {}, Transaction id {}", tx.client, tx.tx)
                    },
                    Err(e) => e
                }
            }
            TransactionType::Chargeback => {
                let client_account = self.client_accounts.get(client_id).cloned();
                let res = client_account.ok_or("Account not found".to_owned())
                    .and_then(|mut account| self.process_chargeback(&tx, &mut account));

                match res {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account.expect("expected client account"));
                        self.past_transactions.push(tx);
                        format!("Resolved. Client id {}, Transaction id {}", tx.client, tx.tx)
                    },
                    Err(e) => e
                }
            }
        }
    }

    fn is_valid_transaction(&self, transaction: &Transaction) -> bool {
        let account = self.client_accounts.get(&transaction.client);
        let transaction_type = transaction.r#type;
        let previous_transaction = self.past_transactions
            .iter()
            .find(|tx| tx.tx == transaction.tx);

        match (account, transaction_type, previous_transaction) {
            (account, TransactionType::Deposit, _) => {
                account.map(|acc| !acc.locked) // is not a valid transaction if acc is locked.
                .or(Some(true)) // if acc doesn't exist we'll create new, so it'll be valid deposit tx.
                .unwrap()
            },
            (Some(account), TransactionType::Withdrawal, _) => {
                if account.available < transaction.amount.expect("Deposit transaction should have valid amount")
                || account.locked {
                    false
                } else {
                    true
                }
            },
            (Some(account), TransactionType::Dispute, Some(prev_tx)) => {
                if prev_tx.r#type != TransactionType::Deposit
                || account.available < prev_tx.amount.expect("amount expected")
                || account.held < prev_tx.amount.expect("amount expected")
                || transaction.client != prev_tx.client
                || account.locked {
                    false
                } else {
                    true
                }
            },
            (Some(account), TransactionType::Resolve, Some(prev_tx)) => {
                if account.held < prev_tx.amount.expect("amount expected")
                || prev_tx.r#type != TransactionType::Deposit
                || transaction.client != prev_tx.client
                || account.locked {
                    false
                } else {
                    true
                }
            },
            (Some(account), TransactionType::Chargeback, Some(prev_tx)) => {
                if account.held < prev_tx.amount.expect("amount expected")
                || transaction.client != prev_tx.client
                || account.locked {
                    false
                } else {
                    true
                }
            }
            (None, _, _) => { // Covers all transaction types except Deposit, w/o existing account
                false
            }
            (_, _, None) => { // Covers all transaction types except Deposit & Withdrawal, w/o prev transaction
                false
            }
        }
    }

    fn process_deposit(&mut self, transaction: &Transaction, client_account: &mut ClientAccount) -> Result<(), String> {
        match transaction.amount {
            Some(amt) => {
                client_account.available += amt;
                client_account.total += amt;
                Ok(())
            }
            None => {
                Err("Amount is not specified".to_owned())
            }
        }
    }

    fn process_withdrawal(&mut self, transaction: &Transaction, client_account: &ClientAccount) -> Result<(), String> {
        let mut client_account = client_account.to_owned();
        match transaction.amount {
            Some(amt) => {
                if client_account.available >= amt {
                    client_account.available -= amt;
                    client_account.total -= amt;
                    Ok(())
                } else {
                    Err("Insufficient available amount to process withdrawal".to_owned())
                }
            }
            None => {
                Err("Amount is not specified".to_owned())
            }
        }
    }

    fn process_dispute(&mut self, transaction: &Transaction, client_account: &mut ClientAccount) -> Result<(), String> {
        let past_transaction = self.past_transactions.iter().find({ |&pt|   
            pt.tx == transaction.tx 
        });
        match past_transaction {
            Some(past_transac) => {
            client_account.available -= past_transac.amount.expect("Transaction amount is None");
            client_account.held += past_transac.amount.expect("Transaction amount is None");
            Ok(())
        }
            None => {
                Err("Dispute Error: Previous transaction not found".to_owned())
            }
        }
    }

    fn process_chargeback(&mut self, transaction: &Transaction, client_account: &mut ClientAccount) -> Result<(), String> {
        let past_transaction = self.past_transactions.iter().find({ |&pt|   
            pt.tx == transaction.tx 
        });
        match past_transaction {
            Some(past_transac) => {
                client_account.held -= past_transac.amount.expect("Transaction amount is None");
                client_account.total -= past_transac.amount.expect("Transaction amount is None");
                client_account.locked = true;
                Ok(())
            },
            None => {
                Err("Chargeback Error: Previous transaction not found".to_owned())
            }
        }
    }
    
    fn process_resolve(&mut self, transaction: &Transaction, client_account: &mut ClientAccount) -> Result<(), String> {
        let past_transaction = self.past_transactions.iter().find({ |&pt|   
            pt.tx == transaction.tx 
        });
        match past_transaction {
            Some(past_transac) => {
                client_account.held -= past_transac.amount.expect("Transaction amount is None");
                client_account.available += past_transac.amount.expect("Transaction amount is None");
                Ok(())
            },
            None => {
                Err("Resolve Error: Previous transaction not found. Invalid transaction id".to_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

#[test]
fn test_is_valid_various_cases() {
    let mut accounts = HashMap::new();
    accounts.insert(1, ClientAccount { total: 200.0, available: 100.0, held: 100.0, locked: false });
    accounts.insert(2, ClientAccount { total: 150.0, available: 100.0, held: 50.0, locked: true });

    let past_transaction = vec![
        Transaction{
            tx: 1,
            client: 1,
            amount: Some(50.0),
            r#type: TransactionType::Deposit,
        },
        Transaction{
            tx: 2,
            client: 1,
            amount: Some(50.0),
            r#type: TransactionType::Deposit,
        },
        Transaction{
            tx: 3,
            client: 2,
            amount: Some(150.0),
            r#type: TransactionType::Deposit,
        },
        Transaction{
            tx: 4,
            client: 2,
            amount: Some(50.0),
            r#type: TransactionType::Withdrawal,
        }
    ];

    let cases = vec![
        (Transaction {
            tx: 10,
            client: 1,
            amount: Some(10.0),
            r#type: TransactionType::Deposit,
        }, true),
        (Transaction {
            tx: 11,
            client: 2,
            amount: Some(20.0),
            r#type: TransactionType::Deposit,
        }, false),
        (Transaction {
            tx: 1,
            client: 1,
            amount: None,
            r#type: TransactionType::Dispute,
        }, true), // Valid tx id for 
        (Transaction {
            tx: 12,
            client: 1,
            amount: None,
            r#type: TransactionType::Dispute,
        }, false), // Invalid tx id for disputing
        (Transaction {
            tx: 1,
            client: 1,
            amount: None,
            r#type: TransactionType::Chargeback,
        }, true), // Valid tx id for chargeback
        (Transaction {
            tx: 1,
            client: 1,
            amount: None,
            r#type: TransactionType::Resolve,
        }, true), // Given valid tx id for resolving
        (Transaction {
            tx: 1,
            client: 2,
            amount: None,
            r#type: TransactionType::Resolve,
        }, false), // Given tx id and client id don't match
        (Transaction {
            tx: 15,
            client: 300,
            amount: Some(100.0),
            r#type: TransactionType::Withdrawal,
        }, false),
        (Transaction {
            tx: 16,
            client: 300,
            amount: Some(100.0),
            r#type: TransactionType::Deposit,
        }, true)
    ];

    let processor = BasicProcessor::new(accounts, past_transaction);

    for (tx, expected) in cases {
        assert_eq!(processor.is_valid_transaction(&tx), expected, "Failed on Tx#{}", tx.tx);
    }
}

}

