use super::errors::TransactionError;

pub type ClientId = u16;

pub type TransactionId = u32;

#[derive(Debug)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
}

#[derive(Debug, PartialEq)]
enum TransactionStatus {
    Valid,
    Disputed,
}

#[derive(Debug)]
pub struct Transaction {
    transaction_type: TransactionType,
    id: TransactionId,
    client_id: ClientId,
    amount: f64,
    status: TransactionStatus,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        id: TransactionId,
        client_id: ClientId,
        amount: f64,
    ) -> Result<Transaction, TransactionError> {
        if amount < 0.0 {
            return Err(TransactionError::AmountIsNegative);
        }

        Ok(Transaction {
            transaction_type,
            id,
            client_id,
            amount: amount.round4(),
            status: TransactionStatus::Valid,
        })
    }

    // Is it better to derive Copy and Clone for the enum instead of returning a reference?
    pub fn transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn dispute(&mut self) -> Result<(), TransactionError> {
        if self.status != TransactionStatus::Valid {
            return Err(TransactionError::AlreadyDisputed);
        }

        self.status = TransactionStatus::Disputed;

        Ok(())
    }
}

pub trait Round4 {
    fn round4(&self) -> f64;
}

impl Round4 for f64 {
    fn round4(&self) -> f64 {
        (*self * 10_000.0).round() / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_amount() {
        let error = Transaction::new(TransactionType::Deposit, 1, 1, -0.0001).unwrap_err();
        assert_eq!(error, TransactionError::AmountIsNegative);
    }

    #[test]
    fn test_round_down() {
        let transaction = Transaction::new(TransactionType::Deposit, 1, 1, 0.00011).unwrap();
        assert_eq!(transaction.amount(), 0.0001);
    }

    #[test]
    fn test_round_up() {
        let transaction = Transaction::new(TransactionType::Deposit, 1, 1, 0.00016).unwrap();
        assert_eq!(transaction.amount(), 0.0002);
    }
}
