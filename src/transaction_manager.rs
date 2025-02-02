use crate::transaction_manager::errors::TransactionError;
use balance::Balance;
use std::collections::HashMap;
pub use transaction::Transaction;
use transaction::{ClientId, TransactionId, TransactionState, TransactionType};

pub mod errors;

mod transaction;

mod balance;

pub struct TransactionManager {
    balances: HashMap<ClientId, Balance>,
    transactions: HashMap<TransactionId, TransactionState>,
}

impl TransactionManager {
    pub fn new() -> TransactionManager {
        TransactionManager {
            balances: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn accept(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        match transaction {
            Transaction::Deposit {
                id,
                client_id,
                amount_base_units: amount,
            } => self.deposit(id, client_id, amount),
            Transaction::Withdrawal {
                id,
                client_id,
                amount_base_units: amount,
            } => self.withdrawal(id, client_id, amount),
            Transaction::Dispute { id, client_id } => self.dispute(id, client_id),
            Transaction::Resolve { id, client_id } => self.resolve(id, client_id),
            Transaction::Chargeback { id, client_id } => self.chargeback(id, client_id),
        }
    }

    fn deposit(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
        amount: i64,
    ) -> Result<(), TransactionError> {
        if let Some(_) = self.transactions.get_mut(&transaction_id) {
            return Err(TransactionError::DuplicateTransaction);
        }

        let transaction_state =
            TransactionState::new(TransactionType::Deposit, transaction_id, client_id, amount)?;

        let balance = self.get_balance_mut(client_id);

        balance.deposit(amount);

        self.insert_transaction(transaction_state);

        Ok(())
    }

    fn withdrawal(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
        amount: i64,
    ) -> Result<(), TransactionError> {
        if let Some(_) = self.transactions.get_mut(&transaction_id) {
            return Err(TransactionError::DuplicateTransaction);
        }

        let transaction_state = TransactionState::new(
            TransactionType::Withdrawal,
            transaction_id,
            client_id,
            amount,
        )?;

        let balance = self.get_balance_mut(client_id);

        balance.withdrawal(amount)?;

        self.insert_transaction(transaction_state);

        Ok(())
    }

    fn dispute(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
    ) -> Result<(), TransactionError> {
        if let Some(disputed_transaction) = self.transactions.get_mut(&transaction_id) {
            if client_id != disputed_transaction.client_id() {
                return Err(TransactionError::DisputeClientMismatch);
            }

            let amount = disputed_transaction.amount();

            disputed_transaction.dispute()?;

            let balance = self.get_balance_mut(client_id);

            balance.hold(amount);

            Ok(())
        } else {
            Err(TransactionError::DisputedTransactionNotFound)
        }
    }

    fn resolve(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
    ) -> Result<(), TransactionError> {
        if let Some(disputed_transaction) = self.transactions.get_mut(&transaction_id) {
            if client_id != disputed_transaction.client_id() {
                return Err(TransactionError::ResolveClientMismatch);
            }

            let amount = disputed_transaction.amount();

            disputed_transaction.resolve()?;

            let balance = self.get_balance_mut(client_id);

            balance.release(amount);

            Ok(())
        } else {
            Err(TransactionError::DisputedTransactionNotFound)
        }
    }

    fn chargeback(
        &mut self,
        transaction_id: TransactionId,
        client_id: ClientId,
    ) -> Result<(), TransactionError> {
        if let Some(disputed_transaction) = self.transactions.get_mut(&transaction_id) {
            if client_id != disputed_transaction.client_id() {
                return Err(TransactionError::ChargebackClientMismatch);
            }

            let amount = disputed_transaction.amount();

            disputed_transaction.chargeback()?;

            let balance = self.get_balance_mut(client_id);

            balance.chargeback(amount);

            Ok(())
        } else {
            Err(TransactionError::DisputedTransactionNotFound)
        }
    }

    fn get_balance_mut(&mut self, client_id: ClientId) -> &mut Balance {
        self.balances.entry(client_id).or_insert(Balance::new())
    }

    fn insert_transaction(&mut self, transaction: TransactionState) {
        let transaction_id = transaction.id();

        if self
            .transactions
            .insert(transaction_id, transaction)
            .is_some()
        {
            // We expect duplicates to have already been checked be we reach here.
            panic!("Duplicate transaction id: {}", transaction_id);
        }
    }

    // Copies balance entries to ClientBalance so as to not break encapsulation.
    pub fn balances(&self) -> Vec<ClientBalance> {
        self.balances
            .iter()
            .map(|(&client_id, balance)| ClientBalance {
                client_id,
                available: ClientBalance::from_base_units(balance.available()),
                held: ClientBalance::from_base_units(balance.held()),
                total: ClientBalance::from_base_units(balance.total()),
                locked: balance.locked(),
            })
            .collect()
    }
}

pub struct ClientBalance {
    pub client_id: ClientId,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl ClientBalance {
    fn from_base_units(amount_base_units: i64) -> f64 {
        amount_base_units as f64 / 10_000.0
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_manager::transaction::TransactionStatus;

    use super::*;

    #[test]
    fn test_deposit() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        assert_eq!(manager.balances[&1].available(), 100);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 100);
        assert_eq!(manager.balances[&1].locked(), false);
        assert_eq!(
            *manager.transactions[&1].transaction_type(),
            TransactionType::Deposit
        );
        assert_eq!(manager.transactions[&1].id(), 1);
        assert_eq!(manager.transactions[&1].client_id(), 1);
        assert_eq!(manager.transactions[&1].amount(), 100);

        // Try another deposit.
        let deposit = Transaction::Deposit {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(deposit).unwrap();

        assert_eq!(manager.balances[&1].available(), 150);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 150);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_deposit_multi_user() {
        let mut manager = TransactionManager::new();

        let deposit1 = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        let deposit2 = Transaction::Deposit {
            id: 2,
            client_id: 2,
            amount_base_units: 200,
        };

        manager.accept(deposit1).unwrap();

        manager.accept(deposit2).unwrap();

        assert_eq!(manager.balances[&1].available(), 100);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 100);
        assert_eq!(manager.balances[&1].locked(), false);
        assert_eq!(
            *manager.transactions[&1].transaction_type(),
            TransactionType::Deposit
        );
        assert_eq!(manager.transactions[&1].id(), 1);
        assert_eq!(manager.transactions[&1].client_id(), 1);
        assert_eq!(manager.transactions[&1].amount(), 100);

        assert_eq!(manager.balances[&2].available(), 200);
        assert_eq!(manager.balances[&2].held(), 0);
        assert_eq!(manager.balances[&2].total(), 200);
        assert_eq!(manager.balances[&2].locked(), false);
        assert_eq!(
            *manager.transactions[&2].transaction_type(),
            TransactionType::Deposit
        );
        assert_eq!(manager.transactions[&2].id(), 2);
        assert_eq!(manager.transactions[&2].client_id(), 2);
        assert_eq!(manager.transactions[&2].amount(), 200);
    }

    #[test]
    fn test_withdrawal() {
        let mut manager = TransactionManager::new();

        // Add funds to the account to give a non-zero balance.
        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        // Withdraw funds from the account.
        let withdrawal = Transaction::Withdrawal {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(withdrawal).unwrap();

        assert_eq!(manager.balances[&1].available(), 50);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 50);
        assert_eq!(manager.balances[&1].locked(), false);
        assert_eq!(
            *manager.transactions[&2].transaction_type(),
            TransactionType::Withdrawal
        );
        assert_eq!(manager.transactions[&2].id(), 2);
        assert_eq!(manager.transactions[&2].client_id(), 1);
        assert_eq!(manager.transactions[&2].amount(), 50);
    }

    #[test]
    fn test_withdrawal_overdraw() {
        let mut manager = TransactionManager::new();

        // Add funds to the account to give a non-zero balance.
        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        // Withdraw funds from the account.
        let withdrawal = Transaction::Withdrawal {
            id: 2,
            client_id: 1,
            amount_base_units: 101,
        };

        let res = manager.accept(withdrawal);

        assert!(matches!(res, Err(TransactionError::InsufficientFunds)));

        assert_eq!(manager.balances[&1].available(), 100);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 100);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_duplicate_deposit() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        // Duplicate
        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        let res = manager.accept(deposit);

        assert!(matches!(res, Err(TransactionError::DuplicateTransaction)));

        assert_eq!(manager.balances[&1].available(), 100);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 100);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_dispute_non_existent_tx() {
        let mut manager = TransactionManager::new();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        let res = manager.accept(dispute);

        assert!(matches!(
            res,
            Err(TransactionError::DisputedTransactionNotFound)
        ));
    }

    #[test]
    fn test_dispute_client_mismatch() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            // Client ID does not match.
            client_id: 2,
        };

        let res = manager.accept(dispute);

        assert!(matches!(res, Err(TransactionError::DisputeClientMismatch)));
    }

    #[test]
    fn test_dispute_withdrawal_fails() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        let withdrawal = Transaction::Withdrawal {
            id: 2,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        manager.accept(withdrawal).unwrap();

        let dispute = Transaction::Dispute {
            // Dispute the withdrawal.
            id: 2,
            client_id: 1,
        };

        let res = manager.accept(dispute);

        assert!(matches!(
            res,
            Err(TransactionError::DisputeWithdrawalNotSupported)
        ));
    }

    #[test]
    fn test_dispute_transaction() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let deposit = Transaction::Deposit {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Disputed
        );

        assert_eq!(manager.balances[&1].available(), 50);
        assert_eq!(manager.balances[&1].held(), 100);
        assert_eq!(manager.balances[&1].total(), 150);
    }

    #[test]
    fn test_resolve_non_existent_tx() {
        let mut manager = TransactionManager::new();

        let resolve = Transaction::Resolve {
            id: 1,
            client_id: 1,
        };

        let res = manager.accept(resolve);

        assert!(matches!(
            res,
            Err(TransactionError::DisputedTransactionNotFound)
        ));
    }

    #[test]
    fn test_resolve_client_mismatch() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        let resolve = Transaction::Resolve {
            id: 1,
            // Client ID does not match.
            client_id: 2,
        };

        let res = manager.accept(resolve);

        assert!(matches!(res, Err(TransactionError::ResolveClientMismatch)));
    }

    #[test]
    fn test_resolve_transaction() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let deposit = Transaction::Deposit {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Disputed
        );

        assert_eq!(manager.balances[&1].available(), 50);
        assert_eq!(manager.balances[&1].held(), 100);
        assert_eq!(manager.balances[&1].total(), 150);

        let resolve = Transaction::Resolve {
            id: 1,
            client_id: 1,
        };

        manager.accept(resolve).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Resolved
        );

        assert_eq!(manager.balances[&1].available(), 150);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 150);
    }

    #[test]
    fn test_chargeback_non_existent_tx() {
        let mut manager = TransactionManager::new();

        let chargeback = Transaction::Chargeback {
            id: 1,
            client_id: 1,
        };

        let res = manager.accept(chargeback);

        assert!(matches!(
            res,
            Err(TransactionError::DisputedTransactionNotFound)
        ));
    }

    #[test]
    fn test_chargeback_client_mismatch() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        let chargeback = Transaction::Chargeback {
            id: 1,
            // Client ID does not match.
            client_id: 2,
        };

        let res = manager.accept(chargeback);

        assert!(matches!(
            res,
            Err(TransactionError::ChargebackClientMismatch)
        ));
    }

    #[test]
    fn test_chargeback_transaction() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let deposit = Transaction::Deposit {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(deposit).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Disputed
        );

        assert_eq!(manager.balances[&1].available(), 50);
        assert_eq!(manager.balances[&1].held(), 100);
        assert_eq!(manager.balances[&1].total(), 150);

        let chargeback = Transaction::Chargeback {
            id: 1,
            client_id: 1,
        };

        manager.accept(chargeback).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Chargeback
        );

        assert_eq!(manager.balances[&1].available(), 50);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), 50);
    }

    #[test]
    fn test_chargeback_transaction_negative_balance() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::Deposit {
            id: 1,
            client_id: 1,
            amount_base_units: 100,
        };

        manager.accept(deposit).unwrap();

        let withdrawal = Transaction::Withdrawal {
            id: 2,
            client_id: 1,
            amount_base_units: 50,
        };

        manager.accept(withdrawal).unwrap();

        let dispute = Transaction::Dispute {
            id: 1,
            client_id: 1,
        };

        manager.accept(dispute).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Disputed
        );

        assert_eq!(manager.balances[&1].available(), -50);
        assert_eq!(manager.balances[&1].held(), 100);
        assert_eq!(manager.balances[&1].total(), 50);

        let chargeback = Transaction::Chargeback {
            id: 1,
            client_id: 1,
        };

        manager.accept(chargeback).unwrap();

        assert_eq!(
            *manager.transactions[&1].status(),
            TransactionStatus::Chargeback
        );

        assert_eq!(manager.balances[&1].available(), -50);
        assert_eq!(manager.balances[&1].held(), 0);
        assert_eq!(manager.balances[&1].total(), -50);
    }
}
