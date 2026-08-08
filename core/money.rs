use crate::error::LedgerError;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Currency {
    NGN,
    USD,
    EUR,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Self {
        Money { amount, currency }
    }

    pub fn value(&self) -> i64 {
        self.amount
    }

    pub fn add(&self, other: &Money) -> Result<Money, LedgerError> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch);
        }

        Ok(Money::new(self.amount + other.amount, self.currency))
    }

    pub fn sub(&self, other: &Money) -> Result<Money, LedgerError> {
        if self.currency != other.currency {
            return Err(LedgerError::CurrencyMismatch);
        }

        Ok(Money::new(self.amount - other.amount, self.currency))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let naira = self.amount / 100;
        let kobo = self.amount.abs() % 100;
        write!(f, "₦{}.{:02}", naira, kobo)
    }
}
