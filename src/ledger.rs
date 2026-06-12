use std::collections::HashMap;

use crate::account::{Account, AccountType};
use crate::error::LedgerError;
use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Ledger {
    accounts: HashMap<String, Account>,
    transactions: Vec<Transaction>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    pub fn create_account(
        &mut self,
        id: &str,
        name: &str,
        account_type: AccountType,
    ) -> Result<&Account, LedgerError> {
        let account = Account::new(id, name, account_type);

        self.accounts.insert(id.to_string(), account);

        Ok(self.accounts.get(id).unwrap())
    }

    pub fn post(&mut self, tx: Transaction) -> Result<String, LedgerError> {
        tx.validate()?;

        let entries = &tx.entries;

        if entries
            .iter()
            .any(|entry| !self.accounts.contains_key(&entry.account_id))
        {
            return Err(LedgerError::AccountNotFound);
        }

        let tx_post_id = tx.id.clone();
        self.transactions.push(tx);

        Ok(tx_post_id)
    }
}
