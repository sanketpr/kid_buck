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
        let existing_client_account = self.client_accounts.get(client_id);

        match existing_client_account {
            Some(account) => {
                if account.is_account_frozen {
                    return "Account is fronzen, transaction declined".to_owned()
                }
            }
            _ => {}
        };

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
                        format!("Withdrew. Client id {}, Transaction id {}, Amount {}", tx.client, tx.tx, tx.amount.unwrap())
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
                        format!("Resolved. Client id {}, Transaction id {}, Amount {}", tx.client, tx.tx, tx.amount.unwrap())
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
                        format!("Resolved. Client id {}, Transaction id {}, Amount {}", tx.client, tx.tx, tx.amount.unwrap())
                    },
                    Err(e) => e
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
                Err("Invalid transaction id".to_owned())
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
                client_account.is_account_frozen = true;
                Ok(())
            },
            None => {
                Err("Invalid transaction id".to_owned())
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
                Err("Invalid transaction id".to_owned())
            }
        }
    }
}

fn _process_delegate<F>(client_accounts: &mut HashMap<ClientId, ClientAccount>,
    transaction: &Transaction,
    process_fn: F) -> Result<(), String>
    where F: Fn(&Transaction, &mut ClientAccount) -> Result<(), String> {
        let client_id = &transaction.client;
        let client_account = client_accounts.get(client_id).cloned();
        let res = client_account.ok_or("Account not found".to_owned())
            .and_then(|mut account| process_fn(&transaction, &mut account));

        match res {
            Ok(()) => {
                client_accounts.insert(*client_id, client_account.expect("expected client account"));
                Ok(())
            },
            Err(e) => Err(e)
        }
}
