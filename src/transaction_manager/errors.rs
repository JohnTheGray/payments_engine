use thiserror::Error;

use super::transaction::TransactionStatus;

#[derive(Error, Debug, PartialEq)]
pub enum TransactionError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Duplicate transaction")]
    DuplicateTransaction,
    #[error("Transaction amount is negative")]
    AmountIsNegative,
    #[error("Invalid transaction state transition: {0} -> {1}")]
    InvalidStateTransition(TransactionStatus, TransactionStatus),
    #[error("Disputed transaction not found")]
    DisputedTransactionNotFound,
    #[error("Dispute does not match client")]
    DisputeClientMismatch,
    #[error("Dispute withdrawal not supported")]
    DisputeWithdrawalNotSupported,
    #[error("Resolve does not match client")]
    ResolveClientMismatch,
    #[error("Chargeback does not match client")]
    ChargebackClientMismatch,
}
