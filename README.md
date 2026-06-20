# LedgerKit

> A small double-entry accounting kernel for fintech teams who want correctness without buying a platform.

LedgerKit is a lightweight, embeddable double-entry ledger engine written in Rust.

It is designed for developers building wallets, payment systems, marketplaces, banking products, gaming economies, escrow services, and other financial applications that require auditable money movement.

Instead of storing balances directly, LedgerKit records every movement of value as immutable journal entries and derives balances from transaction history.

## Why LedgerKit?

Many applications start with a simple approach:

```rust
user.balance += 10000;
```

This works until you need:

* Audit trails
* Chargebacks and reversals
* Reconciliation
* Multi-wallet systems
* Regulatory compliance
* Financial correctness guarantees

LedgerKit solves these problems using double-entry bookkeeping.

## Core Principles

### Double-Entry Accounting

Every transaction must balance.

Example:

| Account               | Debit  | Credit |
| --------------------- | ------ | ------ |
| Cash                  | 10,000 |        |
| User Wallet Liability |        | 10,000 |

The following rule always holds:

```text
Total Debits = Total Credits
```

Unbalanced transactions are rejected.

---

### Immutable History

Transactions are never modified or deleted.

Corrections are performed through reversal transactions.

This guarantees a complete audit trail.

---

### Balance Derivation

Balances are calculated from ledger entries.

Instead of:

```rust
account.balance = 10000;
```

LedgerKit derives balances from transaction history.

This prevents inconsistencies and provides complete traceability.

---

## Features

### Accounts

Create and manage ledger accounts.

Examples:

* Cash
* User Wallet
* Merchant Wallet
* Escrow
* Revenue
* Settlement

### Journal Entries

Represent debits and credits.

```rust
Entry {
    account_id,
    amount,
    entry_type,
}
```

### Transactions

Group multiple entries into a single financial event.

```rust
Transaction {
    id,
    entries,
}
```

### Posting Engine

Validate and record transactions.

```rust
ledger.post(transaction)?;
```

### Balance Computation

Calculate balances from transaction history.

```rust
ledger.balance(account_id)?;
```

### Reversals

Reverse transactions without modifying history.

```rust
ledger.reverse(transaction_id)?;
```

### Audit Trail

Retrieve account history.

```rust
ledger.history(account_id)?;
```

---

## Project Goals

LedgerKit aims to become:

* The SQLite of financial ledgers
* Easy to embed into existing systems
* Correct by default
* Storage agnostic
* Extensible
* Production ready

---

## Architecture

```text
ledger-kit/
├── ledger-core
├── ledger-storage
├── ledger-sqlite
├── ledger-postgres
├── ledger-events
├── examples
├── benches
├── tests
└── docs
```

### ledger-core

Contains:

* Accounts
* Entries
* Transactions
* Validation logic
* Balance calculations

### ledger-storage

Storage abstractions and repository interfaces.

### ledger-sqlite

SQLite implementation.

### ledger-postgres

PostgreSQL implementation.

### ledger-events

Event publishing and subscriptions.

---

## Example

### Create Accounts

```rust
let cash = ledger.create_account("Cash");
let wallet = ledger.create_account("User Wallet");
```

### Record Deposit

```rust
ledger.post(Transaction::new(vec![
    debit(cash, 10_000),
    credit(wallet, 10_000),
]))?;
```

### Query Balance

```rust
let balance = ledger.balance(cash)?;
```

---

## Roadmap

### Phase 1 — Core Ledger

- [x] Account type (Asset, Liability, Equity, Revenue, Expense)
- [x] Money type (i64 cents, Display, Add, Sub)
- [x] Entry type (Debit / Credit with account_id + amount)
- [x] Transaction (batched entries with UUID id)
- [x] Validation (balanced, non-empty enforcement)
- [x] Create accounts
- [x] Post transactions (reject unbalanced / missing account)
- [x] Balance computation (sum of entries for an account)
- [x] Reversal (create offsetting transaction by tx id)
- [x] Transaction count
- [x] Account history (filter transactions involving an account)
- [x] In-memory storage backend
- [x] JSON serialization (serde)

### Phase 2 — Persistence

- [x] Storage trait (`LedgerStore`)
- [x] InMemoryStore (HashMap-backed, full impl)
- [x] SQLiteStore schema (accounts + transactions tables)
- [ ] SQLiteStore `load_transactions` and `load_accounts` (stubbed with `todo!()`)
- [ ] SQLiteStore `save_account` (stubbed)
- [ ] SQLite-backed `Ledger` integration tests

### Phase 3 — Production Features

- [ ] Idempotency keys (duplicate transaction detection)
- [ ] Multi-currency support (currency code on Money)
- [ ] Event publishing (on_post, on_reverse hooks)
- [ ] Transaction metadata (description, timestamp, reference)
- [ ] Entry-level metadata

### Phase 4 — Scale

- [ ] PostgreSQL backend
- [ ] Snapshots / checkpoints for fast balance queries
- [ ] Streaming history (cursor / pagination)
- [ ] High-performance batch balance computation

### Phase 5 — Ecosystem

- [ ] Python bindings (PyO3)
- [ ] Node.js bindings (napi-rs)
- [ ] REST API (axum or actix-web)
- [ ] WASM support
- [ ] CLI tool for inspection

---

## Non-Goals

LedgerKit is not:

* An accounting application
* A bookkeeping UI
* A banking platform
* A payment processor

LedgerKit is a financial infrastructure component.

It provides the ledger layer upon which those systems can be built.

---

## Design Philosophy

### Correctness First

Financial correctness is more important than convenience.

### Explicit Over Implicit

Money movement should always be represented by transactions.

### Immutable Data

History should never be rewritten.

### Small Core

Keep the kernel simple and composable.

---

## Learning Objectives

This project is also designed as a learning challenge for Rust engineers.

By building LedgerKit you will learn:

* Rust ownership and borrowing
* Traits and abstractions
* Error handling
* Domain-driven design
* Repository pattern
* Event-driven architecture
* Database persistence
* Financial systems design
* Testing strategies
* Crate organization

---

## Inspiration

LedgerKit draws inspiration from:

* Traditional accounting systems
* Banking core ledgers
* Event sourcing architectures
* SQLite's embeddable philosophy
* Modern fintech infrastructure platforms

---

## License

MIT

---

## Vision

Financial systems should not start with:

```rust
user.balance += amount;
```

They should start with:

```rust
ledger.post(transaction)?;
```

LedgerKit exists to make financial correctness the default.
