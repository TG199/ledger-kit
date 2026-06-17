use std::collections::HashMap;

use crate::account::{Account, AccountType};
use crate::error::LedgerError;
use crate::transaction::Transaction;
use crate::entry::EntryType;
use crate::money::Money;

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

        let entries = &tx.entries();

        if entries
            .iter()
            .any(|entry| !self.accounts.contains_key(&entry.account_id))
        {
            return Err(LedgerError::AccountNotFound);
        }

        let binding = tx.clone();
        let tx_post_id = binding.id();
        self.transactions.push(tx);

        Ok(tx_post_id.to_string())
    }

    pub fn balance(&self, account_id: &str) -> Result<Money, LedgerError> {
        if !self.accounts.contains_key(account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        let balance = self.transactions
            .iter()
            .flat_map(|tx| tx.entries().iter())
            .filter(|e| e.account_id == account_id)
            .fold(Money::new(0), |acc, entry| {
                match entry.entry_type {
                    EntryType::Debit => acc + entry.amount,
                    EntryType::Credit => acc - entry.amount,
                }
        });

        Ok(balance)
    }

    pub fn reverse(&mut self, tx_id: &str) -> Result<String, LedgerError> {

        let original = self.transanctions
            .iter()
            .filte(|orig_id|);

        if !original {
            return Err(LedgerError::TransactionNotFound);
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountType;
    use crate::entry::Entry;
    use crate::money::Money;
    use crate::transaction::Transaction;

    fn setup_ledger() -> Ledger {
        let mut ledger = Ledger::new();
        ledger.create_account("cash", "Cash Account", AccountType::Asset).unwrap();
        ledger.create_account("wallet", "User Wallet", AccountType::Liability).unwrap();
        ledger
    }

    #[test]
    fn accepts_balanced_transaction() {
        let mut ledger = setup_ledger();

        let tx = Transaction::new("X", vec![
            Entry::debit(
                "cash",
                Money::new(100),
            ),

            Entry::credit(
                "wallet",
                Money::new(100),
            ),
        ]);

        assert!(ledger.post(tx).is_ok());

    }

    #[test]
    fn rejects_unbalanced_transaction() {
        let mut ledger = setup_ledger();

        let tx = Transaction::new("X", vec![
            Entry::debit(
                "cash",
                Money::new(1000),
            ),

            Entry::credit(
                "wallet",
                Money::new(100),
            ),
        ]);

        assert!(matches!(
                ledger.post(tx),
                Err(LedgerError::UnbalancedTransaction)
        ));
    }

    #[test]
    fn rejects_missing_account() {
        let mut ledger = setup_ledger();

        let tx = Transaction::new("X", vec![
            Entry::debit(
                "cash",
                Money::new(100),
            ),
            Entry::credit(
                "does_not_exist",
                Money::new(100),
            ),
        ]);

        assert!(matches!(
                ledger.post(tx), 
                Err(LedgerError::AccountNotFound)
        ));
    }

    #[test]
    fn calculates_balance_after_posting() {
        let mut ledger = setup_ledger();

        let tx = Transaction::new("X", vec![
            Entry::debit(
                "cash",
                Money::new(500),
            ),

            Entry::credit(
                "wallet",
                Money::new(500),
            ),
        ]);

        ledger.post(tx).unwrap();

        let cash_balance = ledger.balance("cash").unwrap();
        let wallet_balance = ledger.balance("wallet").unwrap();

        assert_eq!(cash_balance, Money::new(500));
        assert_eq!(wallet_balance, Money::new(-500));
    }
}
