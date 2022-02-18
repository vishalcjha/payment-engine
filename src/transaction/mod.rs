pub mod validator;

use std::slice::Iter;

use serde::Deserialize;

use self::validator::is_valid_input;


pub enum TransactionType {
    Deposite,
    Withdrawal,
    Dispute,
    Reslove,
    Chargeback
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        use TransactionType::*;
        match self {
            Deposite => "deposit",
            Withdrawal => "withdrawal",
            Dispute => "dispute",
            Reslove => "resolve",
            Chargeback => "chargeback"
        }
    }

    pub fn iterator() -> Iter<'static, TransactionType> {
        use TransactionType::*;
        static TRANSACTION_TYPES: [TransactionType; 5] = [Deposite, Withdrawal, Dispute, Reslove, Chargeback];
        TRANSACTION_TYPES.iter()
    }
}

#[derive(Debug, Deserialize)]
pub enum Transaction {
    Deposit {client_id: u16, transaction_id: u32, amount: f64},
    Withdrawal {client_id: u16, transaction_id: u32, amount: f64},
    DisputedDeposit {client_id: u16, transaction_id: u32, amount: f64},
    DisputedWithdrawal {client_id: u16, transaction_id: u32, amount: f64},
    Dispute {client_id: u16, transaction_id: u32},
    Reslove {client_id: u16, transaction_id: u32},
    Chargeback {client_id: u16, transaction_id: u32},
}

impl Transaction {
    /// This assumes input is valid str that can be converted to Transaction using is_valid_input.
    /// It will panic otherwise.
    pub fn new(input: &str) -> Transaction {
        use Transaction::*;
        assert!(is_valid_input(input));

        let splitted: Vec<&str> = input.split(&[',', ' ']).filter(|each| !each.is_empty()).collect();
        let trans_type = *splitted.get(0).unwrap();
        let client_id = splitted.get(1).unwrap().parse::<u16>().unwrap();
        let transaction_id = splitted.get(2).unwrap().parse::<u32>().unwrap();
        let amount = splitted.get(3).map(|amount| amount.parse::<f64>().unwrap());
        if trans_type.eq("deposit") {
            Deposit {
                client_id,
                transaction_id,
                amount: amount.unwrap(),
            }
        } else if trans_type.eq("withdrawal") {
            Withdrawal {
                client_id,
                transaction_id,
                amount: amount.unwrap(),
            }
        } else if trans_type.eq("dispute") {
            Dispute {
                client_id,
                transaction_id,
            }
        } else if trans_type.eq("resolve") {
            Reslove {
                client_id,
                transaction_id,
            }
        } else if trans_type.eq("chargeback") {
            Chargeback {
                client_id,
                transaction_id,
            }
        } else {
            eprint!("Invalie input {}", input);
            panic!("This should not happen as code has already validated input")
        }
    }

    /// this should only be called for non_refering transcation.
    pub fn make_disputed_transaction(self) -> Result<(Transaction, f64), Transaction>{
        match self {
            Transaction::Deposit { client_id, transaction_id, amount} => Ok((
                Transaction::DisputedDeposit { client_id, transaction_id, amount}, amount)),
            Transaction::Withdrawal { client_id, transaction_id, amount } => Ok((
                Transaction::DisputedWithdrawal { client_id, transaction_id, amount }, amount)),
            _ => Err(self),
        }
    }

    pub fn get_disputed_transaction(self) -> Result<(Transaction, f64), Transaction> {
        match self {
            Transaction::DisputedDeposit { client_id, transaction_id, amount } => Ok((Transaction::Deposit {
                client_id,
                transaction_id,
                amount,
            }, amount)),
            Transaction::DisputedWithdrawal { client_id, transaction_id, amount } => Ok((Transaction::Withdrawal {
                client_id,
                transaction_id,
                amount,
            }, amount)),
            _ => Err(self),
        }
    }

    pub fn is_disputed(&self) -> bool {
        match self {
            Transaction::DisputedDeposit { client_id: _, transaction_id: _, amount: _ }
                | Transaction::DisputedWithdrawal { client_id: _, transaction_id: _, amount: _ } => true,
            _ => false,
        }
    }

    pub fn is_non_refering(&self) -> bool {
        match self {
            Transaction::Deposit { client_id: _, transaction_id: _, amount: _ }
                | Transaction::Withdrawal { client_id: _, transaction_id: _, amount: _ } => true,
            _ => false
        }
    }

    pub fn client_id(&self) -> u16 {
        match self {
            Transaction::Deposit { client_id, transaction_id: _, amount: _ }
            | Transaction::Withdrawal { client_id, transaction_id: _, amount: _ }
            | Transaction::DisputedWithdrawal { client_id, transaction_id: _, amount: _ }
            | Transaction::DisputedDeposit { client_id, transaction_id: _, amount: _ } => *client_id,
            Transaction::Dispute { client_id, transaction_id: _ }
            | Transaction::Reslove { client_id, transaction_id: _ }
            | Transaction::Chargeback { client_id, transaction_id: _ } => *client_id,
        }
    }
}