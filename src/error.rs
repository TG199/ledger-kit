#[derive(Debug, PartialEq)]
pub enum LedgerError {
    UnbalancedTransaction,
    EmptyTransaction,
    AccountNotFound,
    TransactionNotFound,
}
