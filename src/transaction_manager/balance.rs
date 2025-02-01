use super::{errors::TransactionError, transaction::Round4};

#[derive(Debug)]
pub struct Balance {
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Balance {
    pub fn new() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn available(&self) -> f64 {
        self.available
    }

    pub fn held(&self) -> f64 {
        self.held
    }

    pub fn total(&self) -> f64 {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
        // TODO: do we need to round here?
        self.available.round4();
        self.total.round4();
    }

    pub fn withdrawl(&mut self, amount: f64) -> Result<(), TransactionError> {
        // TODO: when comparing floats, can we use guarantee precision?
        if self.available < amount {
            return Err(TransactionError::InsufficientFunds);
        }

        self.available -= amount;
        self.total -= amount;
        // TODO: do we need to round here?
        self.available.round4();
        self.total.round4();

        Ok(())
    }

    pub fn hold(&mut self, amount: f64) {
        // Reduce available balance and increase held balance, but keep total the same.
        self.available -= amount;
        self.held += amount;
    }
}

impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        self.available == other.available
            && self.held == other.held
            && self.total == other.total
            && self.locked == other.locked
    }
}
