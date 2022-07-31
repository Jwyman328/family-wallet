pub mod mocks;

use std::fmt::Error;

use crate::permissions::{BitcoinPermissions};
use crate::HeadOfTheHouse;

#[derive(Debug)]
pub struct Child {
    pub account_id: i32,
    pub account_name: String,
    pub bitcoin_balance: i32,
    pub permissions: Vec<BitcoinPermissions>,
}

impl Child {
    pub fn spend_bitcoin(&self, amount:i32) -> Option<&str>{
       if self.hasPermissionToSpend(){
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

    pub fn askHeadOfHouseToSpendBitcoin(&self, headOfHouse: &HeadOfTheHouse)-> Result<String, String>{
        if self.hasPermissionToSpend(){
            let spent_bitcoin_message = headOfHouse.spend_bitcoin(5);
            return Ok(spent_bitcoin_message)
        }else{
            Err(String::from("No permission to spend."))
        }

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

        assert_eq!(child_with_permissions_to_spend.spend_bitcoin(5), Some("spending_bitcoin"))
    }
    #[test]
    fn child_without_permission_can_not_send_bitcoin(){
        let child_with_permissions_to_spend = get_child_without_permissions_to_spend();

        assert_eq!(child_with_permissions_to_spend.spend_bitcoin(5), None)
    }
}
