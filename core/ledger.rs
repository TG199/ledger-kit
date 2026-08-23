use std::collections::HashMap;

use crate::account::{Account, AccountType};
use crate::entry::{Entry, EntryType};
use crate::error::LedgerError;
use crate::money::Currency;
use crate::money::Money;
use crate::storage::LedgerStore;
use crate::transaction::Transaction;
use crate::event::{Event, EventBus};
use crate::snapshot::AccountSnapshot;

pub struct Ledger<S: LedgerStore> {
    store: S,
    processed_keys: HashMap<String, String>,
    event_bus: EventBus,
    snapshots: HashMap<String, AccountSnapshot>,
}

impl<S: LedgerStore + Default> Ledger<S> {
    pub fn new() -> Self {
        Ledger {
            store: S::default(),
            processed_keys: HashMap::new(),
            event_bus: EventBus::new(),
            snapshots: HashMap::new(),
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
        self.event_bus.publish(&Event::AccountCreated);
        Ok(account)
    }

    pub fn post(
        &mut self,
        tx: Transaction,
        idempotency_key: Option<&str>,
    ) -> Result<String, LedgerError> {
        if let Some(key) = idempotency_key {
            if let Some(existing_id) = self.processed_keys.get(key) {
                return Ok(existing_id.clone());
            }
        }

        tx.validate()?;
        let entries = &tx.entries();
        let accounts = self.store.load_accounts()?;

        if entries
            .iter()
            .any(|entry| !accounts.iter().any(|a| a.id() == entry.account_id))
        {
            return Err(LedgerError::AccountNotFound);
        }

        let tx_post_id = tx.id();

        if let Some(key) = idempotency_key {
            self.processed_keys
                .insert(key.to_string(), tx_post_id.to_string());
        }

        self.store.save_transaction(&tx)?;
        self.event_bus.publish(&Event::TransactionPosted);
        Ok(tx_post_id.to_string())
    }

    pub fn balance(&self, account_id: &str) -> Result<Money, LedgerError> {
        let accounts = self.store.load_accounts()?;
        if !accounts.iter().any(|a| a.id() == account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        let balance = if let Some(snapshot) = self.snapshots.get(account_id) {
            let all_txns = self.store.load_transactions()?;

            let start = all_txns
                .iter()
                .position(|tx| tx.id() == snapshot.last_transaction_id)
                .map(|i| i + 1)
                .unwrap_or(0);

            let relevant = &all_txns[start..];

            relevant
                .iter()
                .flat_map(|tx| tx.entries().iter())
                .filter(|e| e.account_id == account_id)
                .try_fold(snapshot.balance, |acc, entry| {
                    match entry.entry_type {
                        EntryType::Debit => acc.add(&entry.amount),
                        EntryType::Credit=> acc.sub(&entry.amount),
                    }
                })

        } else {
            self.store
                .load_transactions()?
                .iter()
                .flat_map(|tx| tx.entries().iter())
                .filter(|e| e.account_id == account_id)
                .try_fold(Money::new(0, Currency::NGN), |acc, entry| {
                    match entry.entry_type {
                        EntryType::Debit => acc.add(&entry.amount),
                        EntryType::Credit => acc.sub(&entry.amount),
                    }
                })
        };
        Ok(balance?)
    }

    pub fn reverse(
        &mut self,
        tx_id: &str,
        idempotency_key: Option<&str>,
    ) -> Result<String, LedgerError> {
        let original_tx = self
            .store
            .load_transactions()?
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
        let reversed_post = self.post(reversed_tx, idempotency_key);
        self.event_bus.publish(&Event::TransactionReversed);
        reversed_post
    }

    pub fn transaction_count(&self) -> usize {
        self.store.load_transactions().map(|t| t.len()).unwrap_or(0)
    }

    pub fn history(&self, account_id: &str) -> Result<Vec<Transaction>, LedgerError> {
        let accounts = self.store.load_accounts()?;

        if !accounts.iter().any(|a| a.id() == account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        let txns = self
            .store
            .load_transactions()?
            .into_iter()
            .filter(|tx| tx.entries().iter().any(|e| e.account_id == account_id))
            .collect();

        Ok(txns)
    }

    pub fn subscribe(&mut self, handler: Box<dyn Fn(&Event)>) {
        self.event_bus.subscribe(handler)
    }

    pub fn snapshot(&mut self, account_id: &str) -> Result<(), LedgerError> {
        let current_balance = self.balance(&account_id)?;

        let last_transaction_id = self.store.load_transactions()?
            .into_iter()
            .filter(|tx| tx.entries().iter().any(|e| e.account_id == account_id))
            .last()
            .map(|tx| tx.id().to_string())
            .unwrap_or_default();
        let snapshot = AccountSnapshot::new(&account_id, current_balance, &last_transaction_id);
        self.snapshots.insert(account_id.to_string(), snapshot);

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountType;
    use crate::entry::Entry;
    use crate::money::Money;
    use crate::storage::InMemoryStore;
    use crate::transaction::Transaction;
    use crate::money::Currency;
    use std::rc::Rc;
    use std::cell::Cell;

    fn setup_ledger() -> Ledger<InMemoryStore> {
        let mut ledger = Ledger::<InMemoryStore>::new();
        ledger
            .create_account("cash", "Cash Account", AccountType::Asset)
            .unwrap();
        ledger
            .create_account("wallet", "User Wallet", AccountType::Liability)
            .unwrap();
        ledger
    }

    #[test]
    fn accepts_balanced_transaction() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(100, Currency::NGN)),
            Entry::credit("wallet", Money::new(100, Currency::NGN)),
        ]);

        assert!(ledger.post(tx, idempotency_key).is_ok());
    }

    #[test]
    fn rejects_unbalanced_transaction() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(1000, Currency::NGN)),
            Entry::credit("wallet", Money::new(100, Currency::NGN)),
        ]);

        assert!(matches!(
            ledger.post(tx, idempotency_key),
            Err(LedgerError::UnbalancedTransaction)
        ));
    }

    #[test]
    fn rejects_missing_account() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(100, Currency::NGN)),
            Entry::credit("does_not_exist", Money::new(100, Currency::NGN)),
        ]);

        assert!(matches!(
            ledger.post(tx, idempotency_key),
            Err(LedgerError::AccountNotFound)
        ));
    }

    #[test]
    fn calculates_balance_after_posting() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        ledger.post(tx, idempotency_key).unwrap();

        let cash_balance = ledger.balance("cash").unwrap();
        let wallet_balance = ledger.balance("wallet").unwrap();

        assert_eq!(cash_balance, Money::new(500, Currency::NGN));
        assert_eq!(wallet_balance, Money::new(-500, Currency::NGN));
    }

    #[test]
    fn reversal_zeroes_balance() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        ledger.post(tx.clone(), idempotency_key).unwrap();
        let _ = ledger.reverse(tx.id(), Some("x-reversal"));

        let cash_balance = ledger.balance("cash").unwrap();

        assert_eq!(cash_balance, Money::new(0, Currency::NGN));
    }

    #[test]
    fn reversal_creates_new_transaction() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("x");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let tx_id = tx.id().to_string();
        ledger.post(tx, idempotency_key).unwrap();
        let _ = ledger.reverse(&tx_id, Some("x-reversal"));

        assert_eq!(ledger.transaction_count(), 2);
    }

    #[test]
    fn history_returns_relevant_transactions() {
        let mut ledger = setup_ledger();

        let idempotency_key1 = Some("x");
        let idempotency_key2 = Some("y");
        let idempotency_key3 = Some("z");

        let tx1 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("cash", Money::new(500, Currency::NGN)),
        ]);

        let tx2 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("cash", Money::new(500, Currency::NGN)),
        ]);

        let tx3 = Transaction::new(vec![
            Entry::debit("wallet", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        ledger.post(tx1, idempotency_key1).unwrap();
        ledger.post(tx2, idempotency_key2).unwrap();
        ledger.post(tx3, idempotency_key3).unwrap();

        let history = ledger.history("cash");

        assert_eq!(history.unwrap().len(), 2);
    }

    #[test]
    fn transaction_serializes_to_json() {
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(1000, Currency::NGN)),
            Entry::credit("wallet", Money::new(1000, Currency::NGN)),
        ]);
        let json = serde_json::to_string(&tx).unwrap();
        let restored: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(tx.id(), restored.id());
    }

    #[test]
    fn idempotent_post_returns_same_id() {
        let mut ledger = setup_ledger();

        let idempotency_key = Some("Key1");
        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let id1 = ledger.post(tx.clone(), idempotency_key);
        let id2 = ledger.post(tx, idempotency_key);

        assert_eq!(id1, id2);
        assert_eq!(ledger.transaction_count(), 1);
    }

    #[test]
    fn event_published_on_post() {
        let mut ledger = setup_ledger();

        let counter = Rc::new(Cell::new(0u32));
        let idempotency_key = Some("event");

        let counter_clone = Rc::clone(&counter);

        let tx = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        ledger.subscribe(Box::new(move |_event| {
            counter_clone.set(counter_clone.get() + 1);
        }));

        ledger.post(tx, idempotency_key).unwrap();
    

        assert_eq!(1, counter.get())
    }

    #[test]
    fn snapshot_balance_matches_full_replay() {
        let mut ledger = setup_ledger();

        let idempotency_key1 = Some("key1");
        let idempotency_key2 = Some("Key2");
        let idempotency_key3 = Some("Key3");
        let idempotency_key4 = Some("Key4");
        let idempotency_key5 = Some("Key5");

        let tx1 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let tx2 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let tx3 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let tx4 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let tx5 = Transaction::new(vec![
            Entry::debit("cash", Money::new(500, Currency::NGN)),
            Entry::credit("wallet", Money::new(500, Currency::NGN)),
        ]);

        let _ = ledger.post(tx1, idempotency_key1).unwrap();
        let _ = ledger.post(tx2, idempotency_key2);
        let _ = ledger.post(tx3, idempotency_key3);

        let _ = ledger.snapshot("cash");

        let _ = ledger.post(tx4, idempotency_key4);
        let _ = ledger.post(tx5, idempotency_key5);
        assert_eq!(ledger.balance("cash"), Ok(Money::new(2500, Currency::NGN)));
    }
}
