use crate::money::Money;

pub struct AccountSnapshot {
    pub account_id: String,
    pub balance: Money,
    pub last_transaction_id: String,
}

impl AccountSnapshot {
    pub fn new(account_id: &str, balance: Money, last_transaction_id: &str) -> Self{
        Self {
            account_id: account_id.into(),
            balance,
            last_transaction_id: last_transaction_id.into(),
        }
    }
}


