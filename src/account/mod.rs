use std::fmt::Display;

use crate::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Client {
    id: u16,
    available: f64,
    held: f64,
    locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Client {
        Client{
            id,
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }

    pub fn apply_transaction(&mut self, transaction: &Transaction, amount: f64) -> bool {
        if self.locked {
            eprintln!("No Transaction applied for locked account {:?}", self);
            return false;
        }
        match transaction  {
            Transaction::Deposit { client_id: _, transaction_id: _, amount } => {
                self.available += amount;
                true
            },
            Transaction::Withdrawal { client_id: _, transaction_id: _, amount: _ } => {
                if self.available >= amount {
                    self.available -= amount;
                    true
                } else {
                    false
                }
            },
            Transaction::Dispute { client_id: _, transaction_id: _ } => {
                self.available -= amount;
                self.held += amount;
                true
            },
            Transaction::Reslove { client_id: _, transaction_id: _ } => {
                self.available += amount;
                self.held += amount;
                true
            },
            Transaction::Chargeback { client_id: _, transaction_id: _ } => {
                self.available -= amount;
                self.held -= amount;
                self.set_locked(true);
                true
            },
            Transaction::DisputedDeposit { client_id: _, transaction_id: _, amount: _ } 
             | Transaction::DisputedWithdrawal { client_id: _, transaction_id: _, amount: _ } => {
                eprintln!("This transaction {:?} should not come in applyTransaction", transaction);
                false
            },
        }
    }

    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}, {}, {}", self.id, self.available, self.held, self.available + self.held, self.locked)
    }
}