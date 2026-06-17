use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: AccountType,
}

impl Account {
    pub fn new(id: &str, name: &str, account_type: AccountType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            account_type,
        }
    }
}
