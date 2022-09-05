

use crate::permissions::BitcoinPermissions;
use crate::account::Account;
use crate::master_account::MasterAccount;
use crate::children::Children;
use crate::helpers::convert_satoshis_to_float;
use crate::custom_errors::{AccountError, WalletError};
use bdk::{TransactionDetails, Wallet};
use bdk::bitcoin::Address;

// TODO make it so the master account can easily give bitcoin to other accounts, need to give them the entire transactionDetails data
// and address, as if it is their account.

pub struct HeadOfTheHouse {
    pub accounts: Vec<Account>,
    pub master_account: MasterAccount, //all debits and credits, master spending ability of other peoples money
    pub user_id: i32,
}

impl  HeadOfTheHouse {
    pub fn new(mut children: &mut Children, mnemonic_words: Option<String>)-> Result<HeadOfTheHouse, AccountError> {
        let mut head_of_house = HeadOfTheHouse {
            accounts: vec![],
            user_id: 0,
            master_account: MasterAccount::new(mnemonic_words)?

        };
        head_of_house.create_new_user(&mut children, 1, String::from("main"), vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);
        Ok(head_of_house)
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

    pub fn spend_bitcoin(&mut self, user_id: i32, amount: f64, address: &str)-> Result<&'static str, AccountError> {
        // should we be unwrapping here?
        let sufficient_funds = self.does_user_have_sufficient_funds_to_spend(user_id, amount).or_else(|_e| Err(AccountError::InsufficientAccount))?;
        if self.does_user_have_permission_to_spend(user_id) && sufficient_funds {
            // use the master account to spend
            let spend_bitcoin_result = self.master_account.spend_bitcoin(amount, address, 1.0);
            match spend_bitcoin_result {
                Err(_) =>  return Err(AccountError::InsufficientAccount),
                Ok(spend_bitcoin_result) => if spend_bitcoin_result.confirmation_time == None {
                    // update the users account
                    // if transaction still pending add it to the pending list
                    self.add_pending_transaction_to_user_account(user_id, spend_bitcoin_result)?;
                    Ok("PENDING")
                } else {
                    Ok("Success")
                }
            }
        }else{
            return Err(AccountError::InsufficientAccount)
        }
    }

    pub fn does_user_have_sufficient_funds_to_spend(&mut self, user_id:i32, amount_to_spend:f64)->Result<bool, WalletError>{
        let account_balance = self.get_account_balance(user_id)?;
        
        if account_balance > amount_to_spend{
            return Ok(true)
        }else{
            return Ok(false)
        }
    }

    pub fn does_user_have_permission_to_spend(&self, user_id:i32)->bool{
        let user_account_option = self.get_account_by_id(user_id);
        return match user_account_option {
            Some(user_account) => user_account.has_permission_to_spend(),
            None => false
        }
    }

    pub fn subtract_amount_from_user_account(&mut self, user_id:i32, amount: f64)->Result<(), AccountError>{
        let account_option = self.get_mut_account_by_id(user_id);
        match account_option {
            Some(account) => account.subtract_bitcoin_amount(amount),
            None => return Err(AccountError::AccountDoesNotExist("AccountDoesNotExist"))
        };
        Ok(())
    }

    pub fn add_pending_transaction_to_user_account(&mut self, user_id:i32, pending_transaction:TransactionDetails)->Result<(), AccountError>{
        let account_option = self.get_mut_account_by_id(user_id);
        match account_option {
            Some(account) => account.add_pending_transaction(pending_transaction),
            None => return Err(AccountError::AccountDoesNotExist("AccountDoesNotExist"))
        };
        Ok(())
    }

    pub fn get_new_address(&mut self, user_id:i32,)-> Result<Address, WalletError> {
        // get a new address from the master account
        // then add it to the users account
        let new_address = self.master_account.generate_new_address()?;
        // add new address to the users account 
        let account = self.get_mut_account_by_id(user_id).ok_or(WalletError::AddressError)?;
        account.add_address(new_address.clone());
        Ok(new_address)
    }

    pub fn get_account_balance(&mut self, user_id:i32)-> Result<f64, WalletError>{
        let mut total_balance = 0;
        self.master_account.sync_wallet()?;
        let account = self.get_account_by_id(user_id).ok_or(WalletError::AddressError)?;
        let account_script_pub_keys = account.get_addresses_as_script_pub_keys();
        let wallet_utxos = self.master_account.wallet.list_unspent()?;

        for txd in &wallet_utxos{
            // if this address is part of a utxo then add it to the balance
            if account_script_pub_keys.contains(&txd.txout.script_pubkey){
                total_balance += txd.txout.value
            }
         }

        let total_in_float = convert_satoshis_to_float(total_balance);
        let account = self.get_mut_account_by_id(user_id).ok_or(WalletError::AddressError)?;
        account.bitcoin_amount = total_in_float;
        Ok(account.bitcoin_amount)
    }

    pub fn get_pending_spend_amount(&mut self,  user_id:i32)-> Result<f64, AccountError>{ 
        let account_option = self.get_account_by_id(user_id);

        // if no account, throw AccountError
        let account = match account_option {
            Some(account) => account,
            None => return Err(AccountError::AccountDoesNotExist("AccountDoesNotExist"))
        };

        let pending_transactions = &account.pending_transactions;
        
        let mut pending_spend_amount = 0;
        for transaction in pending_transactions{
          pending_spend_amount += transaction.sent;
          pending_spend_amount += transaction.fee.ok_or(AccountError::InsufficientAccount)?;
        }
        Ok(convert_satoshis_to_float(pending_spend_amount))
      }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words, mine_a_block, sleep_while_block_being_mined, get_random_mnenomic_words, test_result_type_is_not_err, get_base_address, set_up, build_mock_transaction};

    // used to handle async await functions
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
        }


    #[test]
    fn get_account_by_id(){
        set_up();
        let mut mock_children = Children::new();
        let mut new_head_of_house = HeadOfTheHouse::new(&mut mock_children, None).unwrap();

        //create child 
        new_head_of_house.create_new_user(&mut mock_children, 2, String::from("one"), vec![BitcoinPermissions::Send]);
        let childs_account = new_head_of_house.get_account_by_id(2).unwrap();
        assert_eq!(childs_account.bitcoin_amount, 0.0);
        assert_eq!(childs_account.account_id, 2);
    }


    #[test]
    fn add_account_automatically_when_adding_new_user() {
        set_up();
        let mut mock_children = Children::new();
        let mut new_head_of_house = HeadOfTheHouse::new(&mut mock_children, None).unwrap();
        new_head_of_house.create_new_user(&mut mock_children,1, String::from("my new user"), vec![BitcoinPermissions::Send]);
        let new_account = new_head_of_house.accounts.get(0).unwrap();
        assert_eq!(new_account.bitcoin_amount, 0.0);
        assert_eq!(new_account.account_id, 1);
    }

    #[test]
    fn test_initiating_new_head_of_house_hold() {
        set_up();
        let mut mock_children = Children::new();
        let new_head_of_house = HeadOfTheHouse::new(&mut mock_children, None).unwrap();
        // automatically create a child for a master
        assert_eq!(mock_children.children.len(), 1);

        let let_head_of_household_master_child = mock_children.get_child_by_id(1).unwrap();
        assert_eq!(let_head_of_household_master_child.user_id, 1);
        assert_eq!(let_head_of_household_master_child.account_name, String::from("main"));


        // automatically create an account for the master
        let let_head_of_household_regular_account = new_head_of_house.get_account_by_id(1).unwrap();
        assert_eq!(let_head_of_household_regular_account.bitcoin_amount, 0.0);
        assert_eq!(let_head_of_household_regular_account.account_id, 1);
        assert_eq!(let_head_of_household_regular_account.permissions, vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);


        // create a master account
        assert_eq!(new_head_of_house.accounts.len(), 1);
        assert_eq!(new_head_of_house.master_account.bitcoin_amount, 0.0);
    }

    #[test]
    fn test_spend_bitcoin_success_from_head_of_house_child_reflected_in_master_account(){
        set_up();
        let  (mut new_head_of_house, children) = set_up_random_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();
        let default_child_address = new_head_of_house.get_new_address(1).unwrap();

        // give the default child some bitcoin
        aw!(mine_a_block(&default_child_address.to_string()));
        sleep_while_block_being_mined();

        // spend the default childs bitcoin
        let spend_result = deafult_child.spend_bitcoin(&mut new_head_of_house,0.5, "bcrt1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6v7cn20");   
        test_result_type_is_not_err(spend_result);

        // put the recent spend btc in a block
        aw!(mine_a_block("crt1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6v7cn20"));
        sleep_while_block_being_mined();

        let master_account_total = new_head_of_house.master_account.get_bitcoin_total();

        // master account should reflect the new total of 2 original btc, and then 1 more and then minus .5 and some fees
        assert_eq!(master_account_total.unwrap() as f64, 149999859.0)
    }

    #[test]
    fn test_spend_bitcoin_unsuccess_from_head_of_house_child_because_insufficient_funds(){
        set_up();
        let  (mut new_head_of_house, children) = set_up_random_user_with_two_bitcoin();
        let deafult_child = children.get_child_by_id(1).unwrap();

        let insuffiecient_funds_error = deafult_child.spend_bitcoin(&mut new_head_of_house,3.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax");
        match insuffiecient_funds_error {
            Err(AccountError::InsufficientAccount) => assert_eq!(true, true), // this is the error it should be 
            _ => assert_eq!(false, true), // if it is any other type of error it should be a failed test
            Ok(_) => assert_eq!(false, true),
        }

        let amount = new_head_of_house.get_account_balance(1).unwrap();
        let master_account_total = new_head_of_house.master_account.get_bitcoin_total().unwrap();

        // user account and master amount should not be deducted
        assert_eq!(amount, 0.0);
        assert_eq!(master_account_total as f64, 100000000.0);
    }

    #[test]
    fn test_child_adding_new_address_adds_to_master_account_then_childs_account(){
        set_up();
        let  (mut new_head_of_house, mut children) = set_up_default_user_with_two_bitcoin();
        // create a second user
        new_head_of_house.create_new_user(&mut children, 2, String::from("user_2"),vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);

        let deafult_child = children.get_child_by_id(1).unwrap();
        let default_child_first_address = deafult_child.get_new_address(&mut new_head_of_house).unwrap();

        let second_child = children.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut new_head_of_house).unwrap();
        
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
        set_up();
        let (mock_children, mut new_head_of_house ) = set_up_user_with_no_bitcoin_and_one_child();

        // get address for child?
        let second_child = mock_children.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut new_head_of_house).unwrap();
        // now send bitcoin to it

        aw!(mine_a_block(&second_child_first_address.to_string()));
        aw!(mine_a_block(&second_child_first_address.to_string()));
        sleep_while_block_being_mined();
        let child_account_balance = new_head_of_house.get_account_balance(2).unwrap();
        
        assert_eq!(child_account_balance, 200000000.0)
    }

    #[test]
    fn test_get_pending_spend_amount_return_pending_spend_values(){
        set_up();
        let (mock_children, mut new_head_of_house ) = set_up_user_with_no_bitcoin_and_one_child();

        // get address for child?
        let second_child = mock_children.get_child_by_id(2).unwrap();
        let second_child_first_address = second_child.get_new_address(&mut new_head_of_house);
        // now send bitcoin to it
        // give the user bitcoin that they can spend
        aw!(mine_a_block(&second_child_first_address.unwrap().to_string()));
        sleep_while_block_being_mined();

        let spend_result = second_child.spend_bitcoin(&mut new_head_of_house, 0.00001, &get_base_address());
        println!("what is the err {:?}", spend_result);
        test_result_type_is_not_err(spend_result);
        
        assert_eq!(new_head_of_house.get_pending_spend_amount(2).unwrap(), 100000141.0)
    }

    #[test]
    fn test_does_user_have_permission_to_spend_get_user_by_id_error_returns_false(){
        set_up();
        let (_mock_children, new_head_of_house ) = set_up_user_with_no_bitcoin_and_one_child();
        let user_id_of_user_that_does_not_exist = 100;
        let does_user_have_permission = new_head_of_house.does_user_have_permission_to_spend(user_id_of_user_that_does_not_exist);
        assert_eq!(does_user_have_permission, false);
    }

    #[test]
    fn test_subtract_amount_from_user_account_unknown_user_return_account_error(){
        set_up();
        let (mock_children, mut new_head_of_house ) = set_up_user_with_no_bitcoin_and_one_child();
        let user_id_of_user_that_does_not_exist = 100;
        let subtract_amount_respone = new_head_of_house.subtract_amount_from_user_account(user_id_of_user_that_does_not_exist, 100.0);
        match subtract_amount_respone {
            Err(AccountError::AccountDoesNotExist(_)) => assert_eq!(true, true), // error should be AccountDoesNotExist
            _ => assert_eq!(true, false), // any other response is false
        }
    }

    #[test]
    fn test_add_pending_transaction_to_user_account_unknown_user_returns_account_error(){
        set_up();
        let (_mock_children, mut new_head_of_house ) = set_up_user_with_no_bitcoin_and_one_child();
        let user_id_of_user_that_does_not_exist = 100;
        let borrowed_master_account = &mut new_head_of_house.master_account;
        let master_account_address = borrowed_master_account.generate_new_address();
        
        // add bitcoin to spending wallet and sync changes before attempting to build the mock transaction
        aw!(mine_a_block(&master_account_address.unwrap().to_string()));
        sleep_while_block_being_mined();
        borrowed_master_account.sync_wallet();
    
        let (_mock_psbt, mock_transaction) = build_mock_transaction(&borrowed_master_account.wallet, 0.00001);
        let subtract_amount_respone = new_head_of_house.add_pending_transaction_to_user_account(user_id_of_user_that_does_not_exist, mock_transaction);
        match subtract_amount_respone {
            Err(AccountError::AccountDoesNotExist(_)) => assert_eq!(true, true), // error should be AccountDoesNotExist
            _ => assert_eq!(true, false), // any other response is false
        }
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
        let mut mock_children = Children::new();
        let mut new_head_of_house = HeadOfTheHouse::new(&mut mock_children, mnemonic_words).unwrap();        
        let default_acconut = new_head_of_house.get_mut_account_by_id(1).unwrap();
        default_acconut.bitcoin_amount = 2.0;

        attach_wallet_to_regtest_electrum_server(&mut new_head_of_house.master_account);

        let master_account_new_address = new_head_of_house.master_account.generate_new_address();

        aw!(mine_a_block(&master_account_new_address.unwrap().to_string()));
        sleep_while_block_being_mined();
        
        (new_head_of_house, mock_children)
    }

    fn set_up_user_with_no_bitcoin_and_one_child()-> (Children, HeadOfTheHouse){
        let mut mock_children = Children::new();
        let mnemonic_words = get_random_mnenomic_words();
        let mut new_head_of_house = HeadOfTheHouse::new(&mut mock_children, mnemonic_words).unwrap();        
        new_head_of_house.get_mut_account_by_id(1).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_head_of_house.master_account);

        new_head_of_house.create_new_user(&mut mock_children, 2, String::from("user_2"),vec![BitcoinPermissions::Send, BitcoinPermissions::Receive]);
        (mock_children, new_head_of_house )
    }
}
