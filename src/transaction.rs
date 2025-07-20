use serde::Deserialize;

pub type ClientId = u32;

#[derive(Debug, Clone, Copy)]
pub struct ClientAccount {
    pub total: f64,
    pub available: f64,
    pub held: f64,
    pub is_account_frozen: bool,
}

impl ClientAccount {
    pub fn new() -> Self {
        ClientAccount {
            total: 0.0,
            available: 0.0,
            held: 0.0,
            is_account_frozen: false
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u32,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}