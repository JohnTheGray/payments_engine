use clap::Parser;
use payments_engine::{
    csv,
    transaction_manager::{ClientBalance, TransactionManager},
};
use std::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let transactions = csv::read_transactions(&args.filename).await?;

    let mut manager = TransactionManager::new();

    for dto in transactions {
        dto.to_transaction()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            .and_then(|tx| {
                manager
                    .accept(tx)
                    .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
            })
            .unwrap_or_else(|err| {
                eprintln!("Ignoring transaction with error: id={} err={}", dto.tx, err)
            });
    }

    let balances = manager.balances();

    print_balances(balances);

    Ok(())
}

// Print the balances CSV to stdout.
fn print_balances(mut balances: Vec<ClientBalance>) {
    // Header
    println!("client,available,held,total,locked");

    // Not necessary, but sorting by client ID for better visual inspection.
    balances.sort_by(|a, b| a.client_id.cmp(&b.client_id));

    // Balances
    for balance in balances {
        println!(
            "{},{},{},{},{}",
            balance.client_id, balance.available, balance.held, balance.total, balance.locked
        );
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    filename: String,
}
