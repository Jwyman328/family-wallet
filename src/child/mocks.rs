use crate::child::Child;
use crate::head_of_the_house::HeadOfTheHouse;
use crate::permissions::BitcoinPermissions;

pub fn get_child_with_permissions_to_spend() -> Child{
    let child_with_permissions = Child {
        account_id: 1,
        account_name: String::from("bob"),
        bitcoin_balance: 0,
        permissions: vec![BitcoinPermissions::Send],
    };
    child_with_permissions
}


pub fn get_child_without_permissions_to_spend() -> Child{
    let child_without_permissions = Child {
        account_id: 1,
        account_name: String::from("bob"),
        bitcoin_balance: 0,
        permissions: vec![],
    };
    child_without_permissions
}