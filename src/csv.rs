use async_stream::try_stream;
use csv_async::AsyncReaderBuilder;
use futures::{Stream, StreamExt};
use serde::Deserialize;
use thiserror::Error;
use tokio::fs;

use crate::transaction_manager::Transaction;

#[derive(Error, Debug)]
pub enum CsvError {
    #[error("Amount is zero or negative")]
    InvalidAmount,
    #[error("Amount is required but is missing")]
    MissingAmount,
}

#[derive(Debug, Deserialize)]
pub enum OrderType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct TransactionDto {
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl TransactionDto {
    pub fn to_transaction(&self) -> Result<Transaction, CsvError> {
        match self.order_type {
            OrderType::Deposit => {
                let amount_base_units;
                if let Some(amount) = self.amount {
                    amount_base_units = Self::to_base_units(amount);
                } else {
                    return Err(CsvError::MissingAmount);
                }

                if amount_base_units <= 0 {
                    Err(CsvError::InvalidAmount)
                } else {
                    Ok(Transaction::Deposit {
                        id: self.tx,
                        client_id: self.client,
                        amount_base_units,
                    })
                }
            }
            OrderType::Withdrawal => {
                let amount_base_units;
                if let Some(amount) = self.amount {
                    amount_base_units = Self::to_base_units(amount);
                } else {
                    return Err(CsvError::MissingAmount);
                }

                if amount_base_units <= 0 {
                    Err(CsvError::InvalidAmount)
                } else {
                    Ok(Transaction::Withdrawal {
                        id: self.tx,
                        client_id: self.client,
                        amount_base_units,
                    })
                }
            }
            OrderType::Dispute => Ok(Transaction::Dispute {
                id: self.tx,
                client_id: self.client,
            }),
            OrderType::Resolve => Ok(Transaction::Resolve {
                id: self.tx,
                client_id: self.client,
            }),
            OrderType::Chargeback => Ok(Transaction::Chargeback {
                id: self.tx,
                client_id: self.client,
            }),
        }
    }

    fn to_base_units(amount: f64) -> i64 {
        (amount * 10_000.0).round() as i64
    }
}

// Read transactions as a stream hence the whole CSV does not need to be in memory at once.
pub fn read_transactions(
    file: fs::File,
) -> impl Stream<Item = Result<TransactionDto, Box<dyn std::error::Error>>> {
    let mut reader = AsyncReaderBuilder::new()
        .has_headers(true)
        .create_deserializer(file);

    try_stream! {
      let mut transactions = reader.deserialize::<TransactionDto>();

      while let Some(transaction) = transactions.next().await {
        yield transaction?;
      }
    }
}
