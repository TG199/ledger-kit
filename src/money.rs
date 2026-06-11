use std::ops::{Add, Sub};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Money(i64);


impl Money {
    pub fn new(amount: i64) -> Self {
        Money(amount)
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Money(self.0 + other.0)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Money(self.0 - other.0)
    }
}


impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let naira = self.0 / 100;
        let kobo  = self.0.abs() % 100;
        write!(f, "₦{}.{:02}", naira, kobo)
    }
}
