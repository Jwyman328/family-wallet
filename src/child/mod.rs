pub mod mocks;

use crate::HeadOfTheHouse;
use crate::custom_errors::{AccountError, WalletError};
use bdk::bitcoin::Address;

/// A Struct representating a Child, which is a user of a wallet.
/// 
/// Each Child has a `user_id` which is associated with the `user_id` of an `Account`
/// The head_of_the_house will coordinate the relationship between a `Child` it's `Account` and the `MasterAccount`.
/// This is why all Child actions are propigated to the `head_of_the_house`.
#[derive(Debug)]
pub struct Child {
    pub user_id: i32,
    pub account_name: String,
}

impl Child {
    /// Spend bitcoin associated with a child's `Account`.
    pub fn spend_bitcoin(&self, head_of_the_house: &mut HeadOfTheHouse, amount:u64, address: &str) -> Result<&'static str, AccountError>{
        head_of_the_house.spend_bitcoin(self.user_id, amount, address)
    }
    pub fn get_new_address(&self, head_of_the_house: &mut HeadOfTheHouse) -> Result<Address, WalletError>{
        let new_address = head_of_the_house.get_new_address(self.user_id)?;
        Ok(new_address)
    }      
}