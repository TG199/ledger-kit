use crate::money::Money;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntryType {
    Debit,
    Credit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub account_id: String,
    pub amount: Money,
    pub entry_type: EntryType,
}

impl Entry {
    pub fn new(account_id: &str, amount: Money, entry_type: EntryType) -> Self {
        Entry {
            account_id: account_id.into(),
            amount,
            entry_type,
        }
    }

    pub fn debit(account_id: &str, amount: Money) -> Self {
        Entry::new(account_id, amount, EntryType::Debit)
    }

    pub fn credit(account_id: &str, amount: Money) -> Self {
        Entry::new(account_id, amount, EntryType::Credit)
    }
}
