use payments_engine::transaction_manager::{
    transaction::Transaction, transaction::TransactionType, TransactionManager,
};

fn main() {
    println!("Hello, world!");

    let mut manager = TransactionManager::new();

    manager
        .accept(Transaction::new(TransactionType::Deposit, 1, 1, 100.0).unwrap())
        .unwrap();
}
