use crate::entry::{Entry, EntryType};
use crate::error::LedgerError;

#[derive(Clone, Debug)]
pub struct Transaction {
    id: String,
    entries: Vec<Entry>,
}

impl Transaction {
    pub fn new(id: &str, entries: Vec<Entry>) -> Self {
        Transaction {
            id: id.into(),
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
            .map(|e| e.amount.value())
            .fold(0i64, |acc, amount| acc + amount);

        let debits_total = self
            .entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Debit)
            .map(|e| e.amount.value())
            .fold(0i64, |acc, amount| acc + amount);

        if credits_total != debits_total {
            return Err(LedgerError::UnbalancedTransaction);
        }

        Ok(())
    }
}
