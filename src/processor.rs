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
        match tx.r#type {
            TransactionType::Deposit => {
                let client_account = self.client_accounts.get_mut(client_id);
                let mut client_account = match client_account {
                    Some(account) => account.to_owned(),
                    None => {
                        let new_account = ClientAccount::new();
                        self.client_accounts.insert(*client_id, new_account.clone());
                        new_account
                    }
                };
                let _ = self.process_deposit(&tx, &mut client_account);
                
                format!("Deposit accepted for Client")
            }
            TransactionType::Withdrawal => {
                let client_account = self.client_accounts.get(client_id).cloned();
                match client_account.ok_or("Account not found".to_owned())
                .and_then(|account| self.process_withdrawal(&tx, &account)) {
                    Ok(()) => {
                        self.client_accounts.insert(*client_id, client_account.expect("expected client account"));
                        format!("Success")
                    },
                    Err(e) => e
                }
            }
            TransactionType::Dispute => {
                let past_transaction = self.past_transactions.iter().find({ |&pt|   
                    pt.tx == tx.tx
                });
    
                let client_account = self.client_accounts.get(&tx.client);
    
                match (past_transaction, client_account) {
                    (Some(past_transac), Some(client_account)) => {
                        let mut client_account = client_account.to_owned();
                        client_account.available -= past_transac.amount.expect("Transaction amount is None");
                        client_account.held += past_transac.amount.expect("Transaction amount is None");
                        self.client_accounts.insert(tx.tx, client_account);
                        format!("Valid transaction not found")
                    }
                    _ => {
                        format!("Invalid transaction id")
                    }
                }
            }
            TransactionType::Resolve => {
                let past_transaction = self.past_transactions.iter().find({ |&pt|   
                    pt.tx == tx.tx
                });
    
                let client_account = self.client_accounts.get(&tx.client);
    
                match (past_transaction, client_account) {
                    (Some(past_transac), Some(client_account)) => {
                        let mut client_account = client_account.to_owned();
                        client_account.available += past_transac.amount.expect("Transaction amount is None");
                        client_account.held -= past_transac.amount.expect("Transaction amount is None");
                        self.client_accounts.insert(tx.tx, client_account);
                        format!("Valid transaction not found")
                    }
                    _ => {
                        format!("Invalid transaction")
                    }
                }
            }
            TransactionType::Chargeback => {
                let past_transaction = self.past_transactions.iter().find({ |&pt|   
                    pt.tx == tx.tx
                });
    
                let client_account = self.client_accounts.get(&tx.client);
    
                match (past_transaction, client_account) {
                    (Some(past_transac), Some(client_account)) => {
                        let mut client_account = client_account.to_owned();
                        client_account.total -= past_transac.amount.expect("Transaction amount is None");
                        client_account.held -= past_transac.amount.expect("Transaction amount is None");
                        client_account.is_account_frozen = true;
                        self.client_accounts.insert(tx.tx, client_account);
                        format!("Valid transaction not found")
                    }
                    _ => {
                        format!("Invalid transaction id.")
                    }
                }
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

    // fn process_dispute(&mut self, transaction: &Transaction, client_account: &mut ClientAccount) -> Result<(), String> {
    //     Ok(())
    // }
}
