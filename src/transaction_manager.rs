use crate::transaction_manager::errors::TransactionError;
use balance::Balance;
use std::collections::HashMap;
use transaction::{ClientId, Transaction, TransactionId, TransactionType};

pub mod errors;

pub mod transaction;

mod balance;

pub struct TransactionManager {
    balances: HashMap<ClientId, Balance>,
    transactions: HashMap<TransactionId, Transaction>,
}

impl TransactionManager {
    pub fn new() -> TransactionManager {
        TransactionManager {
            balances: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn accept(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        if let Some(_) = self.transactions.get_mut(&transaction.id()) {
            return Err(TransactionError::DuplicateTransaction);
        }

        let res = match transaction.transaction_type() {
            TransactionType::Deposit => self
                .deposit(&transaction)
                .and_then(|_| self.insert_transaction(transaction)),
            TransactionType::Withdrawal => self
                .withdrawl(&transaction)
                .and_then(|_| self.insert_transaction(transaction)),
            TransactionType::Dispute => self.dispute(&transaction),
        };

        res
    }

    fn insert_transaction(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        self.transactions.insert(transaction.id(), transaction);
        Ok(())
    }

    fn deposit(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let balance = self.get_balance_mut(transaction.client_id());

        balance.deposit(transaction.amount());

        Ok(())
    }

    fn withdrawl(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let balance = self.get_balance_mut(transaction.client_id());

        balance.withdrawl(transaction.amount())
    }

    fn dispute(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        // TODO: check client ID matches!
        if let Some(disputed_transaction) = self.transactions.get_mut(&transaction.id()) {
            let client_id = disputed_transaction.client_id();
            let amount = disputed_transaction.amount();
            disputed_transaction.dispute()?;

            let balance = self.get_balance_mut(client_id);
            balance.hold(amount);

            return Ok(());
        }

        Err(TransactionError::DisputedNotFound)
    }

    fn get_balance_mut(&mut self, client_id: ClientId) -> &mut Balance {
        self.balances.entry(client_id).or_insert(Balance::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        manager.accept(deposit).unwrap();

        assert_eq!(manager.balances[&1].available(), 100.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 100.0);
        assert_eq!(manager.balances[&1].locked(), false);

        // Try another deposit.
        let deposit = Transaction::new(TransactionType::Deposit, 2, 1, 50.0).unwrap();

        manager.accept(deposit).unwrap();

        assert_eq!(manager.balances[&1].available(), 150.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 150.0);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_deposit_multi_user() {
        let mut manager = TransactionManager::new();

        let deposit1 = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        let deposit2 = Transaction::new(TransactionType::Deposit, 2, 2, 200.0).unwrap();

        manager.accept(deposit1).unwrap();

        manager.accept(deposit2).unwrap();

        assert_eq!(manager.balances[&1].available(), 100.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 100.0);
        assert_eq!(manager.balances[&1].locked(), false);

        assert_eq!(manager.balances[&2].available(), 200.0);
        assert_eq!(manager.balances[&2].held(), 0.0);
        assert_eq!(manager.balances[&2].total(), 200.0);
        assert_eq!(manager.balances[&2].locked(), false);
    }

    #[test]
    fn test_withdrawal() {
        let mut manager = TransactionManager::new();

        // Add funds to the account to give a non-zero balance.
        let deposit = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        manager.accept(deposit).unwrap();

        // Withdraw funds from the account.
        let withdrawal = Transaction::new(TransactionType::Withdrawal, 2, 1, 50.0).unwrap();

        manager.accept(withdrawal).unwrap();

        assert_eq!(manager.balances[&1].available(), 50.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 50.0);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_withdrawal_overdraw() {
        let mut manager = TransactionManager::new();

        // Add funds to the account to give a non-zero balance.
        let deposit = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        manager.accept(deposit).unwrap();

        // Withdraw funds from the account.
        let withdrawal = Transaction::new(TransactionType::Withdrawal, 2, 1, 100.0001).unwrap();

        let res = manager.accept(withdrawal);

        assert!(matches!(res, Err(TransactionError::InsufficientFunds)));

        assert_eq!(manager.balances[&1].available(), 100.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 100.0);
        assert_eq!(manager.balances[&1].locked(), false);
    }

    #[test]
    fn test_duplicate_deposit() {
        let mut manager = TransactionManager::new();

        let deposit = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        manager.accept(deposit).unwrap();

        // Duplicate
        let deposit = Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap();

        let res = manager.accept(deposit);

        assert!(matches!(res, Err(TransactionError::DuplicateTransaction)));

        assert_eq!(manager.balances[&1].available(), 100.0);
        assert_eq!(manager.balances[&1].held(), 0.0);
        assert_eq!(manager.balances[&1].total(), 100.0);
        assert_eq!(manager.balances[&1].locked(), false);
    }
}
