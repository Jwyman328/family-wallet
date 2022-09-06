pub mod mocks;

use crate::HeadOfTheHouse;
use crate::custom_errors::{AccountError, WalletError};
use bdk::bitcoin::Address;

#[derive(Debug)]
pub struct Child {
    pub user_id: i32,
    pub account_name: String,
}

impl Child {
    pub fn spend_bitcoin(&self, head_of_the_house: &mut HeadOfTheHouse, amount:u64, address: &str) -> Result<&'static str, AccountError>{
        head_of_the_house.spend_bitcoin(self.user_id, amount, address)
    }
    pub fn get_new_address(&self, head_of_the_house: &mut HeadOfTheHouse) -> Result<Address, WalletError>{
        let new_address = head_of_the_house.get_new_address(self.user_id)?;
        Ok(new_address)
    }      
}
//#[cfg(test)]
// mod tests {
//     use super::*;
//     use mocks::{mock_child};

//     //#[test]
//     // fn has_permission_to_spend_returns_true_when_child_has_such_permission() {
//     //     let child_with_permissions = get_child_with_permissions_to_spend();

//     //     assert_eq!(child_with_permissions.has_permission_to_spend(), true)  
//     // }
    
   
// }
