use std::{sync::Mutex, collections::HashMap};

use account::Client;
use transaction::Transaction;

pub mod transaction;
pub mod account;

pub trait TransactionEngine {
    fn add_transaction(& mut self, transaction: Transaction) -> bool;
    fn snap_shot_clients(&self) -> Vec<Client>;
}

pub struct InMemoryTransactionEngine {
    tranasctions: Mutex<HashMap<u32, Transaction>>,
    clients: Mutex<HashMap<u16, Client>>,
    // these are transactions applied after client account has been locked.
    // They do not play any role in client account but kept for house keeping,
    // so that can be applied once account in unlocked and audited.
    // not locking it for now as it is used single place for now and that can be accomodated by tranasctions lock.
    blocked_transactions: Vec<Transaction>,
    // once transaction is resolved, it comes here for historical reference.
    // not locking it for now as it is used single place for now and that can be accomodated by tranasctions lock.
    finalized_transactions: Vec<Transaction>,
}

impl InMemoryTransactionEngine {
    pub fn new() -> Self {
        InMemoryTransactionEngine {
            tranasctions: Mutex::new(HashMap::new()),
            clients: Mutex::new(HashMap::new()),
            blocked_transactions: Vec::new(),
            finalized_transactions: Vec::new(),
         }
    }
}
impl TransactionEngine for InMemoryTransactionEngine {
    /// This method add transaction to Engine.
    /// Following are rules
    /// 1. Client Account has to be not in locked state. It will do nothing if account is locked.
    /// 2. Deposit will simply increase available balance.
    /// 3. Withdraw will check if account has more available balance than withdrawal amount, it will let transaction go.
    /// 4. Only Transaction that can be disputed are Deposit or Withdrawal.
    /// 5. Only Disputed Transaction can be 
    ///     a. Resolved - once resolved, transaction is removed from tranasctions,
    ///     otherwise one can keep disputing same transaction and gain system.
    ///     b. Chargeback - once applied, transaction is removed from tranasctions,
    ///     also client account is locked and no further transaction is allowed on client.
    fn add_transaction(&mut self, transaction_to_add: Transaction) -> bool {
        let mut transactions = self.tranasctions.lock().unwrap();
        let mut clients = self.clients.lock().unwrap();

        if let Some(client) = clients.get(&transaction_to_add.client_id()) {
            if client.is_locked() {
                println!("Skipping this transaction as client account is locked {:?}", &transaction_to_add);
                self.blocked_transactions.push(transaction_to_add);
                return false;
            }
        }

        match transaction_to_add {
            Transaction::Deposit { client_id, transaction_id, amount}
                | Transaction::Withdrawal { client_id, transaction_id, amount } => {
                let added = match clients.get_mut(&client_id) {
                    Some(existing_client) => { existing_client.apply_transaction(&transaction_to_add, amount) },
                    None => {
                        let mut client = Client::new(client_id);
                        let added = client.apply_transaction(&transaction_to_add, amount);
                        clients.insert(client_id, client);
                        added
                    },
                };
                if added {
                    transactions.insert(transaction_id, transaction_to_add);
                    true
                } else {
                    false
                }
            }
            Transaction::Dispute { client_id, transaction_id } => {
                if let Some(client) = clients.get_mut(&client_id) {
                    return match transactions.remove(&transaction_id) {
                        Some(existing_transaction) => {
                            match existing_transaction.make_disputed_transaction() {
                                Ok((disputed_transaction, amount)) => {
                                    client.apply_transaction(&transaction_to_add, amount);
                                    transactions.insert(transaction_id, disputed_transaction);
                                },
                                Err(transaction) => {
                                    // non disputable transaction are put back as we removed earlier.
                                    // this can happen when a transaction is disputed twice, and we should keep one.
                                    transactions.insert(transaction_id, transaction);
                                },
                            }
                            true
                        },
                        None => {
                            eprintln!("Skipping {} as not present with engine", transaction_id);
                            false
                        },
                    }
                }
                false
            },
            Transaction::Reslove { client_id, transaction_id }
                | Transaction::Chargeback { client_id, transaction_id } => {
                if let Some(client) = clients.get_mut(&client_id) {
                    return match transactions.remove(&transaction_id) {
                        Some(existing_transaction) if existing_transaction.is_disputed() => {
                            if let Ok((disputed_transaction, amount)) = existing_transaction
                                .get_disputed_transaction() {
                                self.finalized_transactions.push(disputed_transaction);
                                client.apply_transaction(&transaction_to_add, amount);
                            }
                            true
                        },
                        Some(existing_transaction) => {
                            eprintln!("Neglecting {:?} as not disputed transaction", existing_transaction);
                            transactions.insert(transaction_id, existing_transaction);
                            false
                        }
                        None => {
                            eprintln!("Skipping {} as not present with engine", transaction_id);
                            false
                        },
                    }
                }
                false
            },
            _ => {
                eprintln!("This should not come here");
                false
            }
        }
    }

    fn snap_shot_clients(&self) -> Vec<Client> {
        let clients = self.clients.lock().unwrap();
        clients.values().map(|client| client.clone()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_undisputed_transaction_for_resolve_chargeback() {
        let mut engine = InMemoryTransactionEngine::new();
        let deposite_trans = Transaction::new("deposit, 1, 1, 1.0");
        assert!(engine.add_transaction(deposite_trans));

        let resolve_trans = Transaction::new("resolve, 1, 1, 1.0");
        assert!(!engine.add_transaction(resolve_trans));

        let resolve_trans = Transaction::new("chargeback, 1, 1, 1.0");
        assert!(!engine.add_transaction(resolve_trans));

        let disputed_trans = Transaction::new("dispute, 1, 1");
        let resolve_trans = Transaction::new("resolve, 1, 1");
        assert!(engine.add_transaction(disputed_trans));
        assert!(engine.add_transaction(resolve_trans));

        // after above resolve, this transaction should not be active with engine
        let disputed_trans = Transaction::new("dispute, 1, 1");
        assert!(!engine.add_transaction(disputed_trans));
    }

    #[test]
    fn test_charge_back_should_skip_all_future_transaction() {
        let mut engine = InMemoryTransactionEngine::new();
        let deposite_trans = Transaction::new("deposit, 1, 1, 1.0");
        assert!(engine.add_transaction(deposite_trans));

        let disputed_trans = Transaction::new("dispute, 1, 1");
        let resolve_trans = Transaction::new("chargeback, 1, 1");
        assert!(engine.add_transaction(disputed_trans));
        assert!(engine.add_transaction(resolve_trans));

        let deposite_trans = Transaction::new("deposit, 1, 2, 1.0");
        assert!(!engine.add_transaction(deposite_trans));
    }

    #[test]
    fn test_withdrawal_shold_be_skipped_if_low_balance() {
        let mut engine = InMemoryTransactionEngine::new();
        let deposite_trans = Transaction::new("deposit, 1, 1, 1.0");
        assert!(engine.add_transaction(deposite_trans));

        let withdrawal_trans = Transaction::new("withdrawal, 1, 2, 1.1");
        assert!(!engine.add_transaction(withdrawal_trans));

        let disputed_trans = Transaction::new("dispute, 1, 2");
        assert!(!engine.add_transaction(disputed_trans));

        let disputed_trans = Transaction::new("dispute, 1, 1");
        assert!(engine.add_transaction(disputed_trans));
    }
}