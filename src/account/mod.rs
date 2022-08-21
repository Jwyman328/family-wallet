pub mod mocks;
use bdk::bitcoin::Address;


use crate::permissions::BitcoinPermissions;
/// An Account contains a series of permissions, an account_id, and a bitcoin amount
/// the spend_bitcoin function is not complete. is it even used?

#[derive(Debug)]
pub struct Account {
    pub bitcoin_amount: f64,
    pub account_id: i32,
    pub permissions: Vec<BitcoinPermissions>,
    pub addresses: Vec<Address>,
}

impl  Account {
    pub fn new(bitcoin_amount: f64, account_id: i32, permissions: Vec<BitcoinPermissions>)-> Account {
        let new_account = Account {
            bitcoin_amount: bitcoin_amount,
            account_id: account_id,
            permissions: permissions,
            addresses:Vec::new()
        };
        new_account
    }

    pub fn spend_bitcoin(&self, amount:f64) -> Option<&str>{
        if self.hasPermissionToSpend() {
          Some("spending_bitcoin")
        }else {
          None
        }
     }
 
     pub fn hasPermissionToSpend(&self)-> bool{
         let mut hasPermissionToSpend = false;
         for permission in &self.permissions {
             if *permission == BitcoinPermissions::Send{
                 hasPermissionToSpend = true
             }
         }
         hasPermissionToSpend 
     }

     pub fn subtract_bitcoin_amount(&mut self, amount: f64){
        self.bitcoin_amount = self.bitcoin_amount - amount;
     }

     pub fn has_sufficient_funds_to_spend(&self, amount_to_spend: f64)-> bool{
        let has_enough_bitcoin_to_spend = self.bitcoin_amount > amount_to_spend;
        has_enough_bitcoin_to_spend
     }

     pub fn add_address(&mut self, new_address:Address){
        self.addresses.push(new_address);
     }
 
}


mod tests {
    use super::*;
    use mocks::{get_child_with_permissions_to_spend, get_child_without_permissions_to_spend};

    #[test]
    fn hasPermissionToSpend_returns_true_when_child_has_such_permission() {
        let child_with_permissions = get_child_with_permissions_to_spend();

        assert_eq!(child_with_permissions.hasPermissionToSpend(), true)  
    }
    #[test]
    fn hasPermissionToSpend_returns_false_when_child_has_no_such_permission() {
        let child_with_permissions = get_child_without_permissions_to_spend();

        assert_eq!(child_with_permissions.hasPermissionToSpend(), false)  
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

    fn subtract_bitcoin_amount_subtracts(){
        let mut child_with_permissions_to_spend = get_child_with_permissions_to_spend();
        child_with_permissions_to_spend.bitcoin_amount = 2.0;
        child_with_permissions_to_spend.subtract_bitcoin_amount(1.0);

        assert_eq!(child_with_permissions_to_spend.bitcoin_amount, 1.0)
    }
}
