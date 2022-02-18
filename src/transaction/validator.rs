use std::cmp::Ordering;

use super::TransactionType;

pub fn is_valid_input(input: &str) -> bool {
    let splitted: Vec<&str> = input.split(&[',', ' ']).filter(|each| !each.is_empty()).collect();
    if splitted.is_empty() || splitted.len() < 3 {
        return false;
    }
    let trans_type = *splitted.get(0).unwrap();
    let client_id = *splitted.get(1).unwrap();
    let trans_id = *splitted.get(2).unwrap();
    let optional_amount = splitted.get(3);

    if !is_valid_transaction_type(trans_type)
        || !is_valid_client_id(client_id)
        || !is_valid_transaction_id(trans_id) {
            return false;
        }

    if (TransactionType::Deposite.as_str().cmp(trans_type) == Ordering::Equal
        || TransactionType::Withdrawal.as_str().cmp(trans_type) == Ordering::Equal)
        && !optional_amount.map_or(false, |amount| is_valid_amount(*amount)) {
            return false;
        }
        
    true
}

fn is_valid_transaction_type(input_type: &str) -> bool {
    for trans_type in TransactionType::iterator() {
        if trans_type.as_str().cmp(input_type) == Ordering::Equal {
            return true
        }
    }
    false
}

fn is_valid_client_id(id: &str) -> bool {
    id.parse::<u16>().is_ok()
}

fn is_valid_transaction_id(id: &str) -> bool {
    id.parse::<u32>().is_ok()
}

fn is_valid_amount(amount: &str) -> bool {
    amount.parse::<f64>().is_ok()
}