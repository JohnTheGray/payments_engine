# Exploring Rust: Basic Payments Engine

A basic payments engine that handles deposits, withdrawals and disputes.

Transactions are read from the file specified in the arguments, and the client balances are emitted via stdout.
Any non-panic errors are emitted via stderr.

Crates:

- tokio for async.
- csv-async for CSV reader support.
- serde for derserialization support.
- thiserror for easy error types.
- clap for command line argument support.

## Usage

Usage: `cargo run -- transactions.csv > accounts.csv`

Input CSV format:

| type       | client | tx  | amount |
| ---------- | ------ | --- | ------ |
| deposit    | 1      | 1   | 100    |
| withdrawal | 1      | 2   | 10     |
| dispute    | 1      | 2   |        |
| resolve    | 1      | 2   |        |

Ouput CSV format:
| client | available | held | total | locked |
|--------|-----------|-------|----------|--------|
| 1 | 90 | 0 | 90 | false |
| 2 | 100.9999 | 10 | 110.9999 | false |
