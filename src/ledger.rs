use crate::account::{Account, AccountType};
use crate::error::LedgerError;
use crate::transaction::Transaction;
use crate::entry::{Entry, EntryType};
use crate::money::Money;
use crate::storage::LedgerStore;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger<S: LedgerStore> {
    store: S,
}

impl <S: LedgerStore + Default> Ledger<S> {
    pub fn new() -> Self {
        Ledger {
            store: S::default(),
        }
    }

    pub fn create_account(
        &mut self,
        id: &str,
        name: &str,
        account_type: AccountType,
    ) -> Result<Account, LedgerError> {
        let account = Account::new(id, name, account_type);
        self.store.save_account(&account)?;

        Ok(account)
    }

    pub fn post(&mut self, tx: Transaction) -> Result<String, LedgerError> {
        tx.validate()?;

        let entries = &tx.entries();
        let accounts = self.store.load_accounts()?;

        if entries
            .iter()
            .any(|entry| !accounts.iter().any(|a| a.id() == entry.account_id))
        {
            return Err(LedgerError::AccountNotFound);
        }

        let binding = tx.clone();
        let tx_post_id = binding.id();
        self.store.save_transaction(&tx)?;
        Ok(tx_post_id.to_string())
    }

    pub fn balance(&self, account_id: &str) -> Result<Money, LedgerError> {
        let accounts = self.store.load_accounts()?;
        if !accounts.iter().any(|a| a.id() == account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        let balance = self.store.load_transactions()?
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
        let original_tx = self.store.load_transactions()?
            .into_iter()
            .find(|tx| tx.id() == tx_id)
            .ok_or_else(|| LedgerError::TransactionNotFound)?;

        let reversed_entries: Vec<Entry> = original_tx
            .entries()
            .iter()
            .map(|e| match e.entry_type {
                EntryType::Debit => Entry::credit(&e.account_id, e.amount),
                EntryType::Credit => Entry::debit(&e.account_id, e.amount),
            })
        .collect();

        let reversed_tx = Transaction::new(reversed_entries);

        self.post(reversed_tx)
    }

    pub fn transaction_count(&self) -> usize {
        self.store.load_transactions().map(|t| t.len()).unwrap_or(0)
    }

    pub fn history(&self, account_id: &str) -> Result<Vec<Transaction>, LedgerError> {
        let accounts = self.store.load_accounts()?;

        if !accounts.iter().any(|a| a.id() == account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        let txns = self.store.load_transactions()?
            .into_iter()
            .filter(|tx| {
                tx.entries()
                    .iter()
                    .any(|e| e.account_id == account_id)

        })
        .collect();

        Ok(txns)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountType;
    use crate::entry::Entry;
    use crate::money::Money;
    use crate::transaction::Transaction;
    use crate::storage::InMemoryStore;

    fn setup_ledger() -> Ledger::<InMemoryStore> {
        let mut ledger = Ledger::<InMemoryStore>::new();
        ledger.create_account("cash", "Cash Account", AccountType::Asset).unwrap();
        ledger.create_account("wallet", "User Wallet", AccountType::Liability).unwrap();
        ledger
    }

    #[test]
    fn accepts_balanced_transaction() {
        let mut ledger = setup_ledger();

        let tx = Transaction::new(vec![
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

        let tx = Transaction::new(vec![
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

        let tx = Transaction::new(vec![
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

        let tx = Transaction::new(vec![
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

    #[test]
    fn reversal_zeroes_balance() {
        let mut ledger = setup_ledger();

         let tx = Transaction::new(vec![
            Entry::debit(
                "cash",
                Money::new(500),
            ),

            Entry::credit(
                "wallet",
                Money::new(500),
            ),
        ]);

        ledger.post(tx.clone()).unwrap();
        let _ = ledger.reverse(tx.id());

        let cash_balance = ledger.balance("cash").unwrap();

        assert_eq!(cash_balance, Money::new(0));

    }

    #[test]
    fn reversal_creates_new_transaction() {

         let mut ledger = setup_ledger();

         let tx = Transaction::new(vec![
            Entry::debit(
                "cash",
                Money::new(500),
            ),

            Entry::credit(
                "wallet",
                Money::new(500),
            ),
        ]);

         let tx_id = tx.id().to_string();
         ledger.post(tx).unwrap();
         let _ = ledger.reverse(&tx_id);

         assert_eq!(ledger.transaction_count(), 2);
    }

    #[test]
    fn history_returns_relevant_transactions() {
        let mut ledger = setup_ledger();

         let tx1 = Transaction::new(vec![
            Entry::debit(
                "cash",
                Money::new(500),
            ),

            Entry::credit(
                "cash",
                Money::new(500),
            ),
        ]);

        let tx2 = Transaction::new(vec![
            Entry::debit(
                "cash",
                Money::new(500),
            ),

            Entry::credit(
                "cash",
                Money::new(500),
            ),
        ]);

        let tx3 = Transaction::new(vec![
            Entry::debit(
                "wallet",
                Money::new(500),
            ),

            Entry::credit(
                "wallet",
                Money::new(500),
            ),
        ]);

        ledger.post(tx1).unwrap();
        ledger.post(tx2).unwrap();
        ledger.post(tx3).unwrap();

        let history = ledger.history("cash");

        assert_eq!(history.unwrap().len(), 2);
    }

    #[test]
    fn transaction_serializes_to_json() {
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(1000)),
            Entry::credit("wallet", Money::new(1000)),
        ]);
        let json = serde_json::to_string(&tx).unwrap();
        let restored: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(tx.id(), restored.id());
    }
}

