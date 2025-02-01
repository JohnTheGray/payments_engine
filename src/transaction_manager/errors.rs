use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Duplicate transaction")]
    DuplicateTransaction,
    #[error("Transaction amount is negative")]
    AmountIsNegative,
    #[error("Transaction already disputed")]
    AlreadyDisputed,
    #[error("Disputed transaction not found")]
    DisputedNotFound,
}
