
#[derive(Debug)]

pub struct MasterAccount {
    pub bitcoin_amount: i32,
    pub all_addresses: Vec<&'static str>,
    pub account_addresses: Vec<&'static str>,
}

impl MasterAccount {
    pub fn new() -> MasterAccount{
        MasterAccount {
            bitcoin_amount: 0,
            all_addresses: Vec::new(),
            account_addresses: Vec::new()
        }
    }

    pub fn spend_bitcoin(&mut self, amount: i32) -> Result<&'static str, &'static str>{
        // acctually sign and broatcast a transaction here
        self.bitcoin_amount = self.bitcoin_amount - amount;
        println!("you have spent {} bitcoin, you now have {} remaining", amount, self.bitcoin_amount);
        Ok("Success")
    }

    pub fn generate_new_address(&mut self) -> &'static str {
        // TODO a function to generate a real address
        let my_new_address = "MOCK ADDRESS";
        self.all_addresses.push(my_new_address);
        my_new_address
    }
}


pub mod test {
    use super::*;

    #[test]
    fn master_account_initialized_with_no_bitcoin(){
        let new_master_account = MasterAccount::new();

        assert_eq!(new_master_account.bitcoin_amount, 0)
    }
    #[test]
    fn spend_bitcoin_returns_success_and_reduces_bitcoin_amount(){
        let mut new_master_account = MasterAccount::new();

        new_master_account.bitcoin_amount = 2;
        let response = new_master_account.spend_bitcoin(1).unwrap();
        assert_eq!(new_master_account.bitcoin_amount, 1);
        assert_eq!(response, "Success")

        
    }
}