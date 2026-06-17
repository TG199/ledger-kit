use crate::account::Account;
use crate::transaction::Transaction;
use crate::error::LedgerError;

use rusqlite::{Connection, params};

pub trait LedgerStore {
    fn save_transaction(&mut self, tx: &Transaction) -> Result<(), LedgerError>;
    fn load_transactions(&self) -> Result<Vec<Transaction>, LedgerError>;
    fn save_account(&mut self, account: &Account) -> Result<(), LedgerError>;
    fn load_accounts(&self) -> Result<Vec<Account>, LedgerError>;
}


pub struct SQLiteStore {
    conn: Connection,
}

impl SQLiteStore {
    pub fn new(path: &str) -> Result<Self, LedgerError> {
        let conn = Connection::open(path)
            .map_err(|_| LedgerError::StorageError)?;

        conn.exec_batch("
             CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );
            ").map_err(|_| LeedegerError::StorageError)?;

            Ok(SQLiteStore { conn })
    }
}

impl LedgerStore for SQLiteStore {

    fn save_transaction(&mut self, tx: &Transaction) -> Result<(), LedgerError> {
        let data = serde_json::to_string(tx)
            .map_err(|_| LedgerError::StorageError)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO transactions (id, data) VALUES (?1, ?2)",
            params![tx.id(), data],
        ).map_err(|_| LedgerError::StorageError)?;
        Ok(())
    }
}
