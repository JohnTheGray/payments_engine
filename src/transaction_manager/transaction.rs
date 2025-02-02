use super::errors::TransactionError;
use std::fmt;

pub type ClientId = u16;

pub type TransactionId = u32;

#[derive(Debug, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionStatus {
    Valid,
    Disputed,
    Resolved,
    Chargeback,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Valid => write!(f, "Valid"),
            TransactionStatus::Disputed => write!(f, "Disputed"),
            TransactionStatus::Resolved => write!(f, "Resolved"),
            TransactionStatus::Chargeback => write!(f, "Chargeback"),
        }
    }
}

pub enum Transaction {
    Deposit {
        id: TransactionId,
        client_id: ClientId,
        amount_base_units: i64,
    },
    Withdrawal {
        id: TransactionId,
        client_id: ClientId,
        amount_base_units: i64,
    },
    Dispute {
        id: TransactionId,
        client_id: ClientId,
    },
    Resolve {
        id: TransactionId,
        client_id: ClientId,
    },
    Chargeback {
        id: TransactionId,
        client_id: ClientId,
    },
}

#[derive(Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct TransactionState {
    transaction_type: TransactionType,
    id: TransactionId,
    client_id: ClientId,
    amount_base_units: i64,
    status: TransactionStatus,
}

impl TransactionState {
    pub fn new(
        transaction_type: TransactionType,
        id: TransactionId,
        client_id: ClientId,
        amount: i64,
    ) -> Result<TransactionState, TransactionError> {
        if amount < 0 {
            return Err(TransactionError::AmountIsNegative);
        }

        Ok(TransactionState {
            transaction_type,
            id,
            client_id,
            amount_base_units: amount,
            status: TransactionStatus::Valid,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn amount(&self) -> i64 {
        self.amount_base_units
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }

    pub fn dispute(&mut self) -> Result<(), TransactionError> {
        if let TransactionType::Withdrawal = self.transaction_type() {
            // Disputing withdrawals is currently not supported. It is not clear what should happen in this case.
            return Err(TransactionError::DisputeWithdrawalNotSupported);
        }

        if self.status != TransactionStatus::Valid {
            return Err(TransactionError::InvalidStateTransition(
                self.status.clone(),
                TransactionStatus::Disputed,
            ));
        }

        self.status = TransactionStatus::Disputed;

        Ok(())
    }

    pub fn resolve(&mut self) -> Result<(), TransactionError> {
        if self.status != TransactionStatus::Disputed {
            return Err(TransactionError::InvalidStateTransition(
                self.status.clone(),
                TransactionStatus::Resolved,
            ));
        }

        self.status = TransactionStatus::Resolved;

        Ok(())
    }

    pub fn chargeback(&mut self) -> Result<(), TransactionError> {
        if self.status != TransactionStatus::Disputed {
            return Err(TransactionError::InvalidStateTransition(
                self.status.clone(),
                TransactionStatus::Chargeback,
            ));
        }

        self.status = TransactionStatus::Chargeback;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_amount() {
        let error = TransactionState::new(TransactionType::Deposit, 1, 1, -1).unwrap_err();
        assert_eq!(error, TransactionError::AmountIsNegative);
    }

    #[test]
    fn test_dispute_resolve_state_transition() {
        let mut state = TransactionState::new(TransactionType::Deposit, 1, 1, 100).unwrap();

        assert_eq!(state.status, TransactionStatus::Valid);

        state.dispute().unwrap();

        assert_eq!(state.status, TransactionStatus::Disputed);

        state.resolve().unwrap();

        assert_eq!(state.status, TransactionStatus::Resolved);
    }

    #[test]
    fn test_dispute_chargeback_state_transition() {
        let mut state = TransactionState::new(TransactionType::Deposit, 1, 1, 100).unwrap();

        assert_eq!(state.status, TransactionStatus::Valid);

        state.dispute().unwrap();

        assert_eq!(state.status, TransactionStatus::Disputed);

        state.chargeback().unwrap();

        assert_eq!(state.status, TransactionStatus::Chargeback);
    }

    #[test]
    fn test_valid_resolved_fails() {
        let mut state = TransactionState::new(TransactionType::Deposit, 1, 1, 100).unwrap();

        assert_eq!(state.status, TransactionStatus::Valid);

        let res = state.resolve();

        assert!(matches!(
            res,
            Err(TransactionError::InvalidStateTransition(
                TransactionStatus::Valid,
                TransactionStatus::Resolved
            ))
        ));
    }

    #[test]
    fn test_valid_chargeback_fails() {
        let mut state = TransactionState::new(TransactionType::Deposit, 1, 1, 100).unwrap();

        assert_eq!(state.status, TransactionStatus::Valid);

        let res = state.chargeback();

        assert!(matches!(
            res,
            Err(TransactionError::InvalidStateTransition(
                TransactionStatus::Valid,
                TransactionStatus::Chargeback
            ))
        ));
    }
}
