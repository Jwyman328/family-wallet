pub mod mocks;
use bdk::{bitcoin::{Address, Script}, TransactionDetails};

use crate::permissions::BitcoinPermissions;
/// An Account contains a series of permissions, an account_id, and a bitcoin amount
/// the spend_bitcoin function is not complete. is it even used?

#[derive(Debug)]
pub struct Account {
    pub bitcoin_amount: f64,
    pub account_id: i32,
    pub permissions: Vec<BitcoinPermissions>,
    pub addresses: Vec<Address>,
    pub pending_transactions:Vec<TransactionDetails>,
}

impl  Account {
    pub fn new(bitcoin_amount: f64, account_id: i32, permissions: Vec<BitcoinPermissions>)-> Account {
        let new_account = Account {
            bitcoin_amount: bitcoin_amount,
            account_id: account_id,
            permissions: permissions,
            addresses:Vec::new(),
            pending_transactions: Vec::new()
        };
        new_account
    }

    pub fn spend_bitcoin(&self, amount:f64) -> Option<&str>{
        if self.has_permission_to_spend() {
          Some("spending_bitcoin")
        }else {
          None
        }
     }
 
     pub fn has_permission_to_spend(&self)-> bool{
         let mut has_permission_to_spend = false;
         for permission in &self.permissions {
             if *permission == BitcoinPermissions::Send{
                 has_permission_to_spend = true
             }
         }
         has_permission_to_spend 
     }

     pub fn subtract_bitcoin_amount(&mut self, amount: f64){
        self.bitcoin_amount = self.bitcoin_amount - amount;
     }

     pub fn add_address(&mut self, new_address:Address){
        self.addresses.push(new_address);
     }

     pub fn add_pending_transaction(&mut self, pending_transaction: TransactionDetails){
        println!("we are adding a pending transaction {:?}", pending_transaction);
        self.pending_transactions.push(pending_transaction);
     }

     pub fn get_addresses_as_script_pub_keys(&self)-> Vec<Script>{
        let addresses_as_script_pub_keys: Vec<Script> = self.addresses.iter().map(|address| address.script_pubkey()).collect();
        addresses_as_script_pub_keys
     }

 
}

#[cfg(test)]
mod tests {
    use super::*;
    use mocks::{get_child_with_permissions_to_spend, get_child_without_permissions_to_spend};

    #[test]
    fn has_permission_to_spend_returns_true_when_child_has_such_permission() {
        let child_with_permissions = get_child_with_permissions_to_spend();

        assert_eq!(child_with_permissions.has_permission_to_spend(), true)  
    }
    #[test]
    fn has_permission_to_spend_returns_false_when_child_has_no_such_permission() {
        let child_with_permissions = get_child_without_permissions_to_spend();

        assert_eq!(child_with_permissions.has_permission_to_spend(), false)  
    }
    #[test]
    fn child_with_permission_sends_bitcoin_successfully(){
        let child_with_permissions_to_spend = get_child_with_permissions_to_spend();

        assert_eq!(child_with_permissions_to_spend.spend_bitcoin(5.0), Some("spending_bitcoin"))
    }
    #[test]
    fn child_without_permission_can_not_send_bitcoin(){
        let child_with_permissions_to_spend = get_child_without_permissions_to_spend();

        assert_eq!(child_with_permissions_to_spend.spend_bitcoin(5.0), None)
    }

    #[test]
    #[ignore]
    fn subtract_bitcoin_amount_subtracts(){
        let mut child_with_permissions_to_spend = get_child_with_permissions_to_spend();
        child_with_permissions_to_spend.bitcoin_amount = 2.0;
        child_with_permissions_to_spend.subtract_bitcoin_amount(1.0);

        assert_eq!(child_with_permissions_to_spend.bitcoin_amount, 1.0)
    }
}
