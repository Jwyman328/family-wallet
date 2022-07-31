use crate::child::Child;


pub struct Account <'a> {
    bitcoin_amount: i32,
    child_account_id: &'a Child,
}