pub mod mocks;


use crate::permissions::BitcoinPermissions;
/// An Account contains a series of permissions, an account_id, and a bitcoin amount
/// the spend_bitcoin function is not complete. is it even used?

#[derive(Debug)]
pub struct Account {
    pub bitcoin_amount: i32,
    pub account_id: i32,
    pub permissions: Vec<BitcoinPermissions>,
    pub addresses: Vec<&'static str>,
}

impl Account {
    pub fn new(bitcoin_amount: i32, account_id: i32, permissions: Vec<BitcoinPermissions>)-> Account{
        let new_account = Account {
            bitcoin_amount: bitcoin_amount,
            account_id: account_id,
            permissions: permissions,
            addresses:Vec::new()
        };
        new_account
    }

    pub fn spend_bitcoin(&self, amount:i32) -> Option<&str>{
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

     pub fn subtract_bitcoin_amount(&mut self, amount: i32){
        self.bitcoin_amount = self.bitcoin_amount - amount;
     }

     pub fn has_sufficient_funds_to_spend(&self, amount_to_spend: i32)-> bool{
        let has_enough_bitcoin_to_spend = self.bitcoin_amount > amount_to_spend;
        has_enough_bitcoin_to_spend
     }

     pub fn add_address(&mut self, new_address:&'static str){
        self.addresses.push(new_address);
     }
 
    //  pub fn askHeadOfHouseToSpendBitcoin(&self, headOfHouse: &HeadOfTheHouse)-> Result<String, String>{
    //      if self.hasPermissionToSpend(){
    //          let spent_bitcoin_message = headOfHouse.spend_bitcoin(5);
    //          return Ok(spent_bitcoin_message)
    //      }else{
    //          Err(String::from("No permission to spend."))
    //      }
 
    //  }
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

    fn subtract_bitcoin_amount_subtracts(){
        let mut child_with_permissions_to_spend = get_child_with_permissions_to_spend();
        child_with_permissions_to_spend.bitcoin_amount = 2;
        child_with_permissions_to_spend.subtract_bitcoin_amount(1);

        assert_eq!(child_with_permissions_to_spend.bitcoin_amount, 1)
    }
}
