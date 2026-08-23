use crate::account::Account;
use crate::error::LedgerError;
use crate::transaction::Transaction;

use rusqlite::{Connection, params};
use std::collections::HashMap;

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
        let conn = Connection::open(path).map_err(|_| LedgerError::StorageError)?;

        conn.execute_batch(
            "
             CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL
            );
            ",
        )
        .map_err(|_| LedgerError::StorageError)?;

        Ok(SQLiteStore { conn })
    }
}

impl LedgerStore for SQLiteStore {
    fn save_transaction(&mut self, tx: &Transaction) -> Result<(), LedgerError> {
        let data = serde_json::to_string(tx).map_err(|_| LedgerError::StorageError)?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO transactions (id, data) VALUES (?1, ?2)",
                params![tx.id(), data],
            )
            .map_err(|_| LedgerError::StorageError)?;
        Ok(())
    }

    fn load_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        let mut stmt = self.conn
            .prepare("SELECT data FROM transactions")
            .map_err(|_| LedgerError::StorageError)?;

        let items = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })
        .map_err(|_| LedgerError::StorageError)?
        .filter_map(|r| r.ok())
        .filter_map(|data| serde_json::from_str(&data).ok())
        .collect();

        Ok(items)
    }

    fn save_account(&mut self, account: &Account) -> Result<(), LedgerError> {
        let data = serde_json::to_string(account).map_err(|_| LedgerError::StorageError)?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO accounts (id, data) VALUES (?1, ?2)",
                params![account.id(), data],
            )
            .map_err(|_| LedgerError::StorageError)?;
        Ok(())
    }

    fn load_accounts(&self) -> Result<Vec<Account>, LedgerError> {
        let mut stmt = self.conn
            .prepare("SELECT data FROM accounts")
            .map_err(|_| LedgerError::StorageError)?;

        let items = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        })
        .map_err(|_| LedgerError::StorageError)?
        .filter_map(|r| r.ok())
        .filter_map(|data| serde_json::from_str(&data).ok())
        .collect();

        Ok(items)
    }
}

pub struct InMemoryStore {
    transactions: Vec<Transaction>,
    accounts: HashMap<String, Account>,
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerStore for InMemoryStore {
    fn save_transaction(&mut self, tx: &Transaction) -> Result<(), LedgerError> {
        self.transactions.push(tx.clone());
        Ok(())
    }

    fn load_transactions(&self) -> Result<Vec<Transaction>, LedgerError> {
        Ok(self.transactions.clone())
    }

    fn save_account(&mut self, account: &Account) -> Result<(), LedgerError> {
        let id = account.id();
        self.accounts.insert(id.to_string(), account.clone());
        Ok(())
    }

    fn load_accounts(&self) -> Result<Vec<Account>, LedgerError> {
        Ok(self.accounts.values().cloned().collect())
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        InMemoryStore {
            transactions: Vec::new(),
            accounts: HashMap::new(),
        }
    }
}
