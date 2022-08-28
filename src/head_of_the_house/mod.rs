

use crate::child::Child;
use crate::permissions::BitcoinPermissions;
use crate::account::Account;
use crate::master_account::MasterAccount;
use crate::children::Children;
use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words};
use bdk::bitcoin::util::address::Payload;
use bdk::bitcoin::Address;
use bdk::bitcoin::AddressType;
use bdk::bitcoin::util::address::WitnessVersion;

pub struct HeadOfTheHouse {
    pub accounts: Vec<Account>,
    pub master_account: MasterAccount, //all debits and credits, master spending ability of other peoples money
    pub user_id: i32,
}

impl  HeadOfTheHouse {
    pub fn new(mut children: &mut Children, mnemonic_words: Option<String>)-> HeadOfTheHouse {
        let mut head_of_house = HeadOfTheHouse {
            accounts: vec![],
            user_id: 0,
            master_account: MasterAccount::new(mnemonic_words)

        };
        head_of_house.create_new_user(&mut children, 1, String::from("main"), vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);
        head_of_house
    }

    pub fn create_new_user(&mut self, children:&mut Children,  account_id: i32,account_name: String, permissions: Vec<BitcoinPermissions> ){
        children.add_child(account_id, account_name);
        self.add_account(account_id, permissions)
    }

    pub fn add_account(&mut self, account_id:i32, permissions: Vec<BitcoinPermissions>){
        let new_account = Account {
            bitcoin_amount:0.0,
            account_id: account_id,
            permissions: permissions,
            addresses:  vec![]
        };

        self.accounts.push(new_account)
    }


    pub fn get_account_by_id(&self, account_id: i32) -> Option<&Account>{
        for account in &self.accounts {
            if account.account_id == account_id {
                return Some(&account)
            }
        }
        None
    }

    pub fn get_mut_account_by_id(&mut self, account_id: i32) -> Option<& mut Account>{
        for account in &mut self.accounts {
            if account.account_id == account_id {
                return Some(account)
            }
        }
        None
    }

    pub fn spend_bitcoin(&mut self, user_id: i32, amount: f64, address: &str)-> Result<&'static str, &'static str> {
        // should we be unwrapping here?

        if self.does_user_have_permission_to_spend(user_id) && self.does_user_have_sufficient_funds_to_spend(user_id, amount){
            // use the master account to spend
            let spend_bitcoin_result = self.master_account.spend_bitcoin(amount, address, 1.0)?;

            if spend_bitcoin_result == "Success"{
                // update the users account
                self.subtract_amount_from_user_account(user_id, amount);
                // update the master account?
                Ok("Success")
            }else{
                Err("unnecessary error")
            }
        }else{
            return Err("user does not have permission or sufficient funds to spend")
        }
    }

    pub fn does_user_have_sufficient_funds_to_spend(&self, user_id:i32, amount_to_spend:f64)->bool{
        let user_account = self.get_account_by_id(user_id).unwrap();
        user_account.has_sufficient_funds_to_spend(amount_to_spend)
    }

    pub fn does_user_have_permission_to_spend(&self, user_id:i32)->bool{
        let user_account = self.get_account_by_id(user_id).unwrap();
        user_account.hasPermissionToSpend()
    }

    pub fn subtract_amount_from_user_account(&mut self, user_id:i32, amount: f64){
        let account = self.get_mut_account_by_id(user_id).unwrap();
        account.subtract_bitcoin_amount(amount);
    }

    pub fn get_new_address(&mut self, user_id:i32,)-> Address {
        // get a new address from the master account
        // then add it to the users account
        let new_address = self.master_account.generate_new_address();
        // add new address to the users account 
        let account = self.get_mut_account_by_id(user_id).unwrap();
        account.add_address(new_address.clone());
        new_address
    }
}

mod tests {
    use bdk::bitcoin::AddressType;

    use super::*;


    #[test]
    fn get_account_by_id(){
        let mut mockChildren = Children::new();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, None);

        //createChild 
        newHeadOfHouse.create_new_user(&mut mockChildren, 2, String::from("one"), vec![BitcoinPermissions::Send]);
        let childs_account = newHeadOfHouse.get_account_by_id(2).unwrap();
        assert_eq!(childs_account.bitcoin_amount, 0.0);
        assert_eq!(childs_account.account_id, 2);

    }



    #[test]
    fn add_account_automatically_when_adding_new_user() {
        let mut mockChildren = Children::new();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, None);
        newHeadOfHouse.create_new_user(&mut mockChildren,1, String::from("my new user"), vec![BitcoinPermissions::Send]);
        let new_account = newHeadOfHouse.accounts.get(0).unwrap();
        assert_eq!(new_account.bitcoin_amount, 0.0);
        assert_eq!(new_account.account_id, 1);
    }

    #[test]
    fn test_initiating_new_head_of_house_hold() {
        let mut mockChildren = Children::new();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, None);
        // automatically create a child for a master
        assert_eq!(mockChildren.children.len(), 1);

        let let_head_of_household_master_child = mockChildren.get_child_by_id(1).unwrap();
        assert_eq!(let_head_of_household_master_child.user_id, 1);
        assert_eq!(let_head_of_household_master_child.account_name, String::from("main"));


        // automatically create an account for the master
        let let_head_of_household_regular_account = newHeadOfHouse.get_account_by_id(1).unwrap();
        assert_eq!(let_head_of_household_regular_account.bitcoin_amount, 0.0);
        assert_eq!(let_head_of_household_regular_account.account_id, 1);
        assert_eq!(let_head_of_household_regular_account.permissions, vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);


        // create a master account
        assert_eq!(newHeadOfHouse.accounts.len(), 1);
        assert_eq!(newHeadOfHouse.master_account.bitcoin_amount, 0.0);

    }

    #[test]
    fn test_spend_bitcoin_success_from_head_of_house_child(){
        let  (mut new_head_of_house, mut children) = set_up_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();

        deafult_child.spend_bitcoin(&mut new_head_of_house,1.0, "bcrt1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6v7cn20");

        let account = new_head_of_house.get_account_by_id(1).unwrap();

        assert_eq!(account.bitcoin_amount, 1.0)
    }

    #[test]
    fn test_spend_bitcoin_unsuccess_from_head_of_house_child_because_insufficient_funds(){
        let  (mut new_head_of_house, mut children) = set_up_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();

        let insuffiecient_funds_error = deafult_child.spend_bitcoin(&mut new_head_of_house,3.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax");
        match insuffiecient_funds_error {
            Err(m) => assert_eq!(m,"user does not have permission or sufficient funds to spend"),
            Ok(x) => assert_eq!(false, true),
        }

        let account = new_head_of_house.get_account_by_id(1).unwrap();

        // account amount should not be deducted
        assert_eq!(account.bitcoin_amount, 2.0)
    }

    #[test]
    fn test_child_adding_new_address_adds_to_master_account_then_childs_account(){
        let  (mut new_head_of_house, mut children) = set_up_user_with_two_bitcoin();
        // create a second user
        new_head_of_house.create_new_user(&mut children, 2, String::from("user_2"),vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);

        let deafult_child = children.get_child_by_id(1).unwrap();
        let default_child_first_address = deafult_child.get_new_address(&mut new_head_of_house);

        let second_child = children.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut new_head_of_house);

        assert_eq!(default_child_first_address.to_string(), "bcrt1q2t3akpkhr2va4dnha8vzd4jxzp9w2tpw294pqh");
        let default_acconut = new_head_of_house.get_mut_account_by_id(1).unwrap();
        // should not have more than one address in their account
        assert_eq!(default_acconut.addresses.len(), 1);

        assert_eq!(default_acconut.addresses.get(0).unwrap().to_string(),"bcrt1q2t3akpkhr2va4dnha8vzd4jxzp9w2tpw294pqh");
        assert_eq!(new_head_of_house.master_account.all_addresses.get(0).unwrap().to_string(),"bcrt1q2t3akpkhr2va4dnha8vzd4jxzp9w2tpw294pqh");

        assert_eq!(second_child_first_address.to_string(), "bcrt1qadvt5faj05hgld2cyhz4g6zg8kf8rzzska9urp");
        let second_child_acconut = new_head_of_house.get_mut_account_by_id(2).unwrap();
        // should not have more than one address in their account
        assert_eq!(second_child_acconut.addresses.len(), 1);


        assert_eq!(second_child_acconut.addresses.get(0).unwrap().to_string(),"bcrt1qadvt5faj05hgld2cyhz4g6zg8kf8rzzska9urp");
        assert_eq!(new_head_of_house.master_account.all_addresses.get(1).unwrap().to_string(),"bcrt1qadvt5faj05hgld2cyhz4g6zg8kf8rzzska9urp");

        // should be two total address in the master account
        assert_eq!(new_head_of_house.master_account.all_addresses.len(), 2)
    }


    //setup function 
    fn set_up_user_with_two_bitcoin() -> (HeadOfTheHouse, Children){
        let mut mockChildren = Children::new();
        let mnemonic_words = get_default_mnenomic_words();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, mnemonic_words);        
        let default_acconut = newHeadOfHouse.get_mut_account_by_id(1).unwrap();
        default_acconut.bitcoin_amount = 2.0;

        attach_wallet_to_regtest_electrum_server(&mut newHeadOfHouse.master_account);
        
        newHeadOfHouse.master_account.bitcoin_amount = 2.0;
        (newHeadOfHouse, mockChildren)
    }
}
