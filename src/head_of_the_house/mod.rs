

use crate::permissions::BitcoinPermissions;
use crate::account::Account;
use crate::master_account::MasterAccount;
use crate::children::Children;
use crate::helpers::convert_satoshis_to_float;
use bdk::{TransactionDetails};
use bdk::bitcoin::Address;
// TODO make it so the master account can easily give bitcoin to other accounts
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
            addresses:  vec![],
            pending_transactions: vec![],
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

            // if transaction still pending add it to the pending list
            if spend_bitcoin_result.confirmation_time == None {
                // update the users account
                self.add_pending_transaction_to_user_account(user_id, spend_bitcoin_result);
                Ok("PENDING")
            } else {
                Ok("Success")
            }
        }else{
            return Err("user does not have permission or sufficient funds to spend")
        }
    }

    pub fn does_user_have_sufficient_funds_to_spend(&mut self, user_id:i32, amount_to_spend:f64)->bool{
        if self.get_account_balance(user_id) > amount_to_spend{
            return true
        }else{
            return false
        }
    }

    pub fn does_user_have_permission_to_spend(&self, user_id:i32)->bool{
        let user_account = self.get_account_by_id(user_id).unwrap();
        user_account.hasPermissionToSpend()
    }

    pub fn subtract_amount_from_user_account(&mut self, user_id:i32, amount: f64){
        let account = self.get_mut_account_by_id(user_id).unwrap();
        account.subtract_bitcoin_amount(amount);
    }

    pub fn add_pending_transaction_to_user_account(&mut self, user_id:i32, pending_transaction:TransactionDetails){
        let account = self.get_mut_account_by_id(user_id).unwrap();
        account.add_pending_transaction(pending_transaction);
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

    pub fn get_account_balance(&mut self, user_id:i32)-> f64{
        let mut total_balance = 0;
        self.master_account.sync_wallet();
        let account = self.get_account_by_id(user_id).unwrap();
        let account_script_pub_keys = account.get_addresses_as_script_pub_keys();
        let wallet_utxos = self.master_account.wallet.list_unspent().unwrap();

        for txd in &wallet_utxos{
            // if this address is part of a utxo then add it to the balance
            if account_script_pub_keys.contains(&txd.txout.script_pubkey){
                total_balance += txd.txout.value
            }
         }

        let total_in_float = convert_satoshis_to_float(total_balance);
        let account = self.get_mut_account_by_id(user_id).unwrap();
        account.bitcoin_amount = total_in_float;
        account.bitcoin_amount
    }

    pub fn get_pending_spend_amount(&mut self,  user_id:i32)-> f64{ 
        let account = self.get_account_by_id(user_id).unwrap();

        let pending_transactions = &account.pending_transactions;
        
        let mut pending_spend_amount = 0;
        for transaction in pending_transactions{
          pending_spend_amount += transaction.sent;
          pending_spend_amount += transaction.fee.unwrap()
        }
        convert_satoshis_to_float(pending_spend_amount)
      }
}

mod tests {
    use bdk::bitcoin::AddressType;
    use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words, mine_a_block, sleep_while_block_being_mined, get_random_mnenomic_words};

    use crate::master_account;

    use super::*;

    // used to handle async await functions
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
        }


    #[test]
    fn get_account_by_id(){
        let mut mockChildren = Children::new();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, None);

        //create child 
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
    fn test_spend_bitcoin_success_from_head_of_house_child_reflected_in_master_account(){
        let  (mut new_head_of_house, mut children) = set_up_random_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();
        let default_child_address = new_head_of_house.get_new_address(1);

        // give the default child some bitcoin
        aw!(mine_a_block(&default_child_address.to_string()));
        sleep_while_block_being_mined();

        // spend the default childs bitcoin
        deafult_child.spend_bitcoin(&mut new_head_of_house,0.5, "bcrt1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6v7cn20");

        // put the recent spend btc in a block
        aw!(mine_a_block("crt1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6v7cn20"));
        sleep_while_block_being_mined();

        let master_account_total = new_head_of_house.master_account.get_bitcoin_total();

        // master account should reflect the new total of 2 original btc, and then 1 more and then minus .5 and some fees
        assert_eq!(master_account_total.unwrap() as f64, 249999859.0)
    }

    #[test]
    fn test_spend_bitcoin_unsuccess_from_head_of_house_child_because_insufficient_funds(){
        let  (mut new_head_of_house, mut children) = set_up_random_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();

        let insuffiecient_funds_error = deafult_child.spend_bitcoin(&mut new_head_of_house,3.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax");
        match insuffiecient_funds_error {
            Err(m) => assert_eq!(m,"user does not have permission or sufficient funds to spend"),
            Ok(x) => assert_eq!(false, true),
        }

        let amount = new_head_of_house.get_account_balance(1);
        let master_account_total = new_head_of_house.master_account.get_bitcoin_total().unwrap();

        // user account and master amount should not be deducted
        assert_eq!(amount, 0.0);
        assert_eq!(master_account_total as f64, 200000000.0);

    }

    #[test]
    fn test_child_adding_new_address_adds_to_master_account_then_childs_account(){
        let  (mut new_head_of_house, mut children) = set_up_default_user_with_two_bitcoin();
        // create a second user
        new_head_of_house.create_new_user(&mut children, 2, String::from("user_2"),vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);

        let deafult_child = children.get_child_by_id(1).unwrap();
        let default_child_first_address = deafult_child.get_new_address(&mut new_head_of_house);

        let second_child = children.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut new_head_of_house);
        
        let default_acconut = new_head_of_house.get_mut_account_by_id(1).unwrap();
        // should not have more than one address in their account
        assert_eq!(default_acconut.addresses.first().unwrap().to_string(), default_child_first_address.to_string());


        let second_child_acconut = new_head_of_house.get_mut_account_by_id(2).unwrap();
        // should not have more than one address in their account
        assert_eq!(second_child_acconut.addresses.first().unwrap().to_string(), second_child_first_address.to_string());

        // should be two total address in the master account that are equal to the address we have created
        assert_eq!(new_head_of_house.master_account.all_addresses.get(1).unwrap().to_string(), default_child_first_address.to_string());
        assert_eq!(new_head_of_house.master_account.all_addresses.get(2).unwrap().to_string(), second_child_first_address.to_string())

    }

    #[test]
    fn test_get_account_balance_returns_current_account_amount(){
        let (mut mockChildren, mut newHeadOfHouse ) = set_up_user_with_no_bitcoin_and_one_child();

        // get address for child?
        let second_child = mockChildren.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut newHeadOfHouse);
        // now send bitcoin to it

        aw!(mine_a_block(&second_child_first_address.to_string()));
        aw!(mine_a_block(&second_child_first_address.to_string()));
        sleep_while_block_being_mined();
        let child_account_balance = newHeadOfHouse.get_account_balance(2);
        
        assert_eq!(child_account_balance, 200000000.0)
    }

    #[test]
    fn test_get_pending_spend_amount_return_pending_spend_values(){
        let (mut mockChildren, mut newHeadOfHouse ) = set_up_user_with_no_bitcoin_and_one_child();

        // get address for child?
        let second_child = mockChildren.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut newHeadOfHouse);
        // now send bitcoin to it
        // give the user bitcoin that they can spend
        aw!(mine_a_block(&second_child_first_address.to_string()));
        sleep_while_block_being_mined();

        second_child.spend_bitcoin(&mut newHeadOfHouse, 0.5, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw");
        
        assert_eq!(newHeadOfHouse.get_pending_spend_amount(2), 100000141.0)
    }


    //setup function 
    fn set_up_default_user_with_two_bitcoin() -> (HeadOfTheHouse, Children){
        let mnemonic_words = get_default_mnenomic_words();
        set_up_user_with_two_bitcoin(mnemonic_words)
    }

    fn set_up_random_user_with_two_bitcoin() -> (HeadOfTheHouse, Children){
        let mnemonic_words = get_random_mnenomic_words();
        set_up_user_with_two_bitcoin(mnemonic_words)
    }

    fn set_up_user_with_two_bitcoin(mnemonic_words: Option<String>) -> (HeadOfTheHouse, Children){
        let mut mockChildren = Children::new();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, mnemonic_words);        
        let default_acconut = newHeadOfHouse.get_mut_account_by_id(1).unwrap();
        default_acconut.bitcoin_amount = 2.0;

        attach_wallet_to_regtest_electrum_server(&mut newHeadOfHouse.master_account);

        let master_account_new_address = newHeadOfHouse.master_account.generate_new_address();

        aw!(mine_a_block(&master_account_new_address.to_string()));
        aw!(mine_a_block(&master_account_new_address.to_string()));
        sleep_while_block_being_mined();
        
        (newHeadOfHouse, mockChildren)
    }

    fn set_up_user_with_no_bitcoin_and_one_child()-> (Children, HeadOfTheHouse){
        let mut mockChildren = Children::new();
        let mnemonic_words = get_random_mnenomic_words();
        let mut newHeadOfHouse = HeadOfTheHouse::new(&mut mockChildren, mnemonic_words);        
        let default_acconut = newHeadOfHouse.get_mut_account_by_id(1).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut newHeadOfHouse.master_account);

        newHeadOfHouse.create_new_user(&mut mockChildren, 2, String::from("user_2"),vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);
        (mockChildren, newHeadOfHouse )
    }
}
