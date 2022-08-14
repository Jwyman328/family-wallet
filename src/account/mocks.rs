use crate::account::Account;
use crate::head_of_the_house::HeadOfTheHouse;
use crate::permissions::BitcoinPermissions;

pub fn get_child_with_permissions_to_spend() -> Account{
    let account_with_permissions = Account {
        account_id: 1,
        bitcoin_amount: 0,
        permissions: vec![BitcoinPermissions::Send],
        addresses:  vec![]
    };
    account_with_permissions
}


pub fn get_child_without_permissions_to_spend() -> Account{
    let account_without_permissions = Account {
        account_id: 1,
        bitcoin_amount: 0,
        permissions: vec![],
        addresses:  vec![]
    };
    account_without_permissions
}