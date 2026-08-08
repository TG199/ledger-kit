use crate::entry::{Entry, EntryType};
use crate::error::LedgerError;
use crate::money::Currency;
use crate::money::Money;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    id: String,
    entries: Vec<Entry>,
}

impl Transaction {
    pub fn new(entries: Vec<Entry>) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(),
            entries,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn validate(&self) -> Result<(), LedgerError> {
        if self.entries.len() < 2 {
            return Err(LedgerError::EmptyTransaction);
        }

        let credits_total = self
            .entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Credit)
            .map(|e| e.amount)
            .try_fold(Money::new(0, Currency::NGN), |acc, amount| acc.add(&amount));

        let debits_total = self
            .entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Debit)
            .map(|e| e.amount)
            .try_fold(Money::new(0, Currency::NGN), |acc, amount| acc.add(&amount));

        if credits_total != debits_total {
            return Err(LedgerError::UnbalancedTransaction);
        }

        Ok(())
    }
}
