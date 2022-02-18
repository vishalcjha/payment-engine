use std::{fs::File,
    io::{BufReader, BufRead}};

use clap::{Command, Arg};
use payment_engine::{
    InMemoryTransactionEngine,
    TransactionEngine,
    transaction::{validator::is_valid_input, Transaction}};

fn main() {
    let matches = Command::new("Payment Engine")
        .arg(
            Arg::new("file").index(1).required(true)
        )
        .get_matches();
    let transaction_file_name = matches.value_of("file").unwrap();
    let transaction_file = File::open(transaction_file_name).unwrap();
    let transaction_reader = BufReader::new(transaction_file);

    let mut transaction_engine = InMemoryTransactionEngine::new();
    
    for transaction in transaction_reader.lines() {
        if let Ok(transaction) = transaction {
            if !is_valid_input(&transaction) {
                continue;
            }
            let transaction = Transaction::new(&transaction);
            transaction_engine.add_transaction(transaction);
        }
    }

    println!("{}", "client,available,held,total,locked");
    for client in transaction_engine.snap_shot_clients() {
        println!("{}", client);
    }
}
