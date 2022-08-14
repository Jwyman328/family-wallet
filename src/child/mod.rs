pub mod mocks;

use std::fmt::Error;

use crate::permissions::{BitcoinPermissions};
use crate::HeadOfTheHouse;

#[derive(Debug)]
pub struct Child {
    pub user_id: i32,
    pub account_name: String,
}

impl Child {
    pub fn spend_bitcoin(&self, head_of_the_house: &mut HeadOfTheHouse, amount:i32) -> Result<&'static str, &'static str>{
        head_of_the_house.spend_bitcoin(self.user_id, amount)
    }
    pub fn get_new_address(&self, head_of_the_house: &mut HeadOfTheHouse) -> &'static str{
        let new_address = head_of_the_house.get_new_address(self.user_id);
        new_address
    }      
}

// mod tests {
//     use super::*;
//     use mocks::{mock_child};

//     //#[test]
//     // fn hasPermissionToSpend_returns_true_when_child_has_such_permission() {
//     //     let child_with_permissions = get_child_with_permissions_to_spend();

//     //     assert_eq!(child_with_permissions.hasPermissionToSpend(), true)  
//     // }
    
   
// }
