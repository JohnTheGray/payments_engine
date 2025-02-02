use super::errors::TransactionError;

#[derive(Debug)]
pub struct Balance {
    available_base_units: i64,
    held_base_units: i64,
    total_base_units: i64,
    locked: bool,
}

impl Balance {
    pub fn new() -> Self {
        Self {
            available_base_units: 0,
            held_base_units: 0,
            total_base_units: 0,
            locked: false,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn available(&self) -> i64 {
        self.available_base_units
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn held(&self) -> i64 {
        self.held_base_units
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn total(&self) -> i64 {
        self.total_base_units
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn deposit(&mut self, amount: i64) {
        self.available_base_units += amount;

        self.total_base_units += amount;
    }

    pub fn withdrawal(&mut self, amount: i64) -> Result<(), TransactionError> {
        if self.available_base_units < amount {
            return Err(TransactionError::InsufficientFunds);
        }

        self.available_base_units -= amount;

        self.total_base_units -= amount;

        Ok(())
    }

    pub fn hold(&mut self, amount: i64) {
        // Reduce available balance and increase held balance, but keep total the same.
        self.available_base_units -= amount;
        self.held_base_units += amount;
    }

    pub fn release(&mut self, amount: i64) {
        // Increase available balance and decrease held balance, but keep total the same.
        self.available_base_units += amount;
        self.held_base_units -= amount;
    }

    pub fn chargeback(&mut self, amount: i64) {
        // Both the total and held are reduced by the chargeback amount.
        // Note that here we can have a negative available balance without held funds to offset,
        // hence the client could owe us money. Ut seems to be coming in banking, hence we'll]
        // implement here.
        self.total_base_units -= amount;
        self.held_base_units -= amount;
        self.locked = true;
    }
}

impl PartialEq for Balance {
    fn eq(&self, other: &Self) -> bool {
        self.available_base_units == other.available_base_units
            && self.held_base_units == other.held_base_units
            && self.total_base_units == other.total_base_units
            && self.locked == other.locked
    }
}
