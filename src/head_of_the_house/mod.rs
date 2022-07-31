

use crate::child::Child;
use crate::permissions::BitcoinPermissions;
use crate::account::Account;

#[derive(Debug)]
pub struct HeadOfTheHouse {
    pub children: Vec<Child>,
    // pub accounts: Vec<Account>
}

impl HeadOfTheHouse {
    pub fn new()-> HeadOfTheHouse{
        HeadOfTheHouse {
            children: vec![]
        }
    }

    pub fn add_child(&mut self, account_id: i32,account_name: String, permissions: Vec<BitcoinPermissions> ) {
        let new_child = Child {
            account_id: account_id,
            account_name: account_name,
            bitcoin_balance: 0,
            permissions: permissions,
        };
       self.children.push(new_child);
    }

    pub fn spend_bitcoin(&self, amount: i32)-> String {
        String::from("message signed")
    }

    pub fn get_child_by_id(&self, account_id: i32) -> Option<&Child>{
        for child in &self.children {
            if child.account_id == account_id {
                return Some(&child)
            }
        }

        None
    }
}

mod tests {
    use super::*;

    #[test]
    fn get_child_by_id() {
        let mut newHeadOfHouse = HeadOfTheHouse::new();

        //createChild 
        newHeadOfHouse.add_child(1, String::from("one"), vec![BitcoinPermissions::Send]);
        let child = newHeadOfHouse.get_child_by_id(1).unwrap();
        assert_eq!(child.account_id, 1)

    }

    #[test]
    fn have_child_sign_message() {
        let mut newHeadOfHouse = HeadOfTheHouse::new();

        //createChild 
        newHeadOfHouse.add_child(1, String::from("one"), vec![BitcoinPermissions::Send]);
        let child = newHeadOfHouse.get_child_by_id(1);

        
    }

}
