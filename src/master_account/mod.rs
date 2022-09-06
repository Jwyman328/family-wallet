use bdk::{miniscript, Wallet, KeychainKind, SyncOptions, SignOptions};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bdk::wallet::AddressInfo;
use bdk::bitcoin::{Address, Network, Txid};
use bdk::FeeRate;
use bdk::keys::{DerivableKey, GeneratableKey, GeneratedKey, ExtendedKey, bip39::{Mnemonic, WordCount, Language}};
use bdk::template::Bip84;
use std::env;
use std::str::FromStr;
use bdk::electrum_client::Client;
use bdk::blockchain::ElectrumBlockchain;
use bdk::blockchain::Blockchain;
use bdk::TransactionDetails;
use crate::helpers::{convert_float_to_satoshis, convert_satoshis_to_float};
use crate::custom_errors::{WalletError, AccountError};
use bdk::{Error};



pub struct MasterAccount {
    pub bitcoin_amount: u64,
    pub all_addresses: Vec<AddressInfo>,
    pub account_addresses: Vec<AddressInfo>,
    pub wallet: Wallet<MemoryDatabase>,
    pub blockchain: Option<ElectrumBlockchain>,
    pub pending_transactions:Vec<TransactionDetails>,
    pub amount_transfered_to_children: u64
}

impl MasterAccount {
    pub fn new(words:Option<String>) -> Result<MasterAccount, AccountError> {
        let wallet = MasterAccount::generate_wallet(words).map_err(
            |_err| AccountError::InsufficientAccount
        )?;

        Ok(MasterAccount {
            bitcoin_amount: 0,
            all_addresses: Vec::new(),
            account_addresses: Vec::new(),
            wallet: wallet,
            blockchain: None,
            pending_transactions: vec![],
            amount_transfered_to_children: 0,
        })
    }

    pub fn generate_wallet(words:Option<String>)-> Result<Wallet<MemoryDatabase>, WalletError> {
        // if provided words, then use them to generate a wallet,
        // if not then generate your own randomly
        let mnemonic_words = match words {
            Some(w) => w,
            _ => {
                let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).expect("mnemonic unable to be generated");
                // Convert mnemonic to string
                let mnemonic_words = mnemonic.to_string();
                mnemonic_words
            },
        };

        println!("my mnemonic words {}", mnemonic_words);

        let network = Network::Regtest; // Or this can be Network::Bitcoin, Network::Signet or Network::Testnet

        // Parse a mnemonic
        let mnemonic  = Mnemonic::parse(&mnemonic_words)?;
        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).ok_or(WalletError::KeyError)?;

        let wallet = Wallet::new(
            Bip84(xprv, KeychainKind::External),
            Some(Bip84(xprv, KeychainKind::Internal)),
            network,
            MemoryDatabase::default(),
        ).expect("Error instantiating wallet");

        Ok(wallet)
    }

    /// This function connects to a currently running electrum server 
    /// and returns the ElectrumBlockchain struct. 
    /// this can be an running electrum server, by default we will connect to blockstreams
    /// but it could be a local server as well like a regtest one created from nigiri at 127.0.0.1:50000
    pub fn sync_wallet_with_electrum_server(&mut self, electrum_url: Option<&str>) -> Result<(), WalletError>{
        let default_electrum_server = env::var("electrum_server")?;
        let electrum_client_url = electrum_url.unwrap_or(&default_electrum_server);
        
        let client = Client::new(electrum_client_url)?; 
        let blockchain = ElectrumBlockchain::from(client);
        
        self.wallet.sync(&blockchain, SyncOptions::default())?;
        self.set_blockchain(blockchain);
        Ok(())
    }

    pub fn set_blockchain(&mut self, blockchain: ElectrumBlockchain){
        self.blockchain = Some(blockchain);
    }

    pub fn spend_bitcoin(&mut self, amount: u64, address: &str, sat_per_vb: f32 ) -> Result<TransactionDetails, WalletError>{
        // make sure our wallet is up to date before we make a spend.
        self.sync_wallet()?;

        let receiving_address = Address::from_str(address)?;
        let mut tx_builder = self.wallet.build_tx();
        tx_builder
            .add_recipient(receiving_address.script_pubkey(), amount)
            .enable_rbf().fee_rate(FeeRate::from_sat_per_vb(sat_per_vb));

        let (mut psbt, tx_details) = tx_builder.finish()?;

        println!("tx_details is {:?}", tx_details);
    
        self.wallet.sign(&mut psbt, SignOptions::default())?;
        
        // now broadcast it 
        let raw_transaction = psbt.extract_tx();
        let txid = raw_transaction.txid();
        println!("the txid {}", txid);
        let my_blockchain = self.blockchain.as_ref();
        
        let electrum_blockchain_option = my_blockchain;
        

        let _electrum_blockchain = match electrum_blockchain_option {
            Some(electrum_blockchain) => electrum_blockchain.broadcast(&raw_transaction),
            None => return Err(WalletError::BroadcastTransactionError),
        };

        self.sync_wallet()?;

        // copy the transaction so that we can return a clone
        let copied_transaction = TransactionDetails { 
            transaction: tx_details.transaction.clone(),
             txid: tx_details.txid.clone(), 
             received:tx_details.received.clone(),
             sent:  tx_details.sent.clone(), 
             fee: tx_details.fee.clone(), 
             confirmation_time: tx_details.confirmation_time.clone()};

        // now we have a pending transaction, so add it to the list of pending_transactions
        println!("I am pushing tx_details  {:?} to pending_transactions", copied_transaction);
        self.pending_transactions.push(tx_details);

        println!("bitcoin amount {} other amount {}", self.bitcoin_amount, amount);
        // TODO should this be get_bitcoin_total? should we update here?
        // yes we need to update after spending
        self.bitcoin_amount = self.get_bitcoin_total()?; // - amount;
        println!("you have spent {} bitcoin, you now have {} remaining", amount, self.bitcoin_amount);

        Ok(copied_transaction)
    }

    pub fn generate_new_address(&mut self) -> Result<Address, WalletError> {        
        let my_new_address = self.wallet.get_address(New)?;
        let copied_address = my_new_address.clone();
        self.all_addresses.push(my_new_address);
        Ok(copied_address)
    }

    pub fn get_pending_spend_amount(&mut self)-> Result<u64, WalletError>{ 
      let pending_transactions = self.get_pending_transactions()?;
      
      let mut pending_spend_amount = 0;
      for transaction in pending_transactions{
        pending_spend_amount += transaction.sent;
        pending_spend_amount += transaction.fee.ok_or(WalletError::SyncElectrumError)?;
      }
      Ok(pending_spend_amount)
    }

    pub fn get_bitcoin_total(&self)-> Result<u64, Error> { 
        self.sync_wallet().unwrap();
        self.wallet.get_balance()
     }

    pub fn get_bitcoin_total_minus_transfers_to_children(&self) ->Result<u64, Error> {
        let total_amount_from_utxos =  self.get_bitcoin_total()?;
        Ok(total_amount_from_utxos -  self.amount_transfered_to_children)
    }

    pub fn transfer_bitcoin(&mut self, amount: u64){
        self.amount_transfered_to_children += amount
    }

    pub fn receive_bitcoin_transfer_from_child(&mut self, amount: u64){
        self.amount_transfered_to_children -= amount
    }

    pub fn get_pending_transactions(&mut self) -> Result<&Vec<TransactionDetails>, WalletError>{
        // resync the blockchain to the wallet again to get the latest data.
        self.sync_wallet()?;

        // for each pending transaction go check if it is still pending
        let mut transactions_that_are_no_longer_pending: Vec<Txid> = vec![];
        for transaction_detail in &self.pending_transactions{
            let my_transaction  = self.wallet.get_tx(&transaction_detail.txid, false)?.ok_or(WalletError::SyncElectrumError)?;
            
            if my_transaction.confirmation_time != None{
                // remove it from the list if it has been confirmed
                transactions_that_are_no_longer_pending.push(my_transaction.txid.clone());
            }
            println!("txId {} this is my transaction {:?}", transaction_detail.txid, my_transaction);
        }
        // filter out the pending transactions, removing the ones that have been confirmed
        for item in transactions_that_are_no_longer_pending{
            let index = self.pending_transactions.iter().position(|tx_detail| tx_detail.txid.to_string() == item.to_string()).ok_or(WalletError::SyncElectrumError)?;
            self.pending_transactions.remove(index);
        }
        Ok(self.pending_transactions.as_ref())
    }

    pub fn sync_wallet(&self)-> Result<(),WalletError>{
        let current_blockchain_option = self.blockchain.as_ref();
        match current_blockchain_option {
            None => Err(WalletError::SyncElectrumError),
            Some(current_blockchain) => {
                self.wallet.sync(current_blockchain, SyncOptions::default())?;
                Ok(())
            }
        }
        
    }
}

#[cfg(test)]
pub mod test {
    use crate::testing_helpers::{get_random_mnenomic_words, test_result_type_is_not_err, set_up, get_base_address};

    use super::*;
    use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words, get_default_mnenomic_words_2, mine_a_block, sleep_while_block_being_mined};
    
    #[allow(unused_imports)]
    use futures::executor::block_on;
    use tokio_test;

    // used to handle async await functions
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
        }

    #[test]
    fn master_account_initialized_with_no_bitcoin(){
        set_up();
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);

        assert_eq!(new_master_account.bitcoin_amount, 0)
    }
    #[test]
    fn spend_bitcoin_returns_success_and_reduces_bitcoin_amount(){
        set_up();
        let mock_mnemonic = get_random_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        sleep_while_block_being_mined();


        let response = new_master_account.spend_bitcoin(convert_float_to_satoshis(1.0), "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax", 1.0);
        test_result_type_is_not_err(response);
        assert_eq!(new_master_account.bitcoin_amount, 99999790);
    }

    // #[test]
    // TODO finish this test.
    // fn balance_should_reflect_current_amount(){
    //     let mock_mnemonic = get_default_mnenomic_words();

    //     let mut new_master_account = MasterAccount::new(mock_mnemonic);
    //     attach_wallet_to_regtest_electrum_server(&mut new_master_account);
    //     new_master_account.get_bitcoin_total()
    // }
    #[test]
    fn get_pending_transactions_has_one_tx_after_low_fee_transaction(){
        set_up();

        // // use get_default_mnenomic_words_2 so that you hae a fresh wallet not connected to the other wallet derived from get_default_mnenomic_words
        let mock_mnemonic = get_default_mnenomic_words_2();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);

        let spent_transaction = new_master_account.spend_bitcoin(convert_float_to_satoshis(1.0), &get_base_address(), 1.0);
        test_result_type_is_not_err(spent_transaction);

        let pending_transactions = new_master_account.get_pending_transactions().expect("get_pending_transactions_has_one_tx_after_low_fee_transaction pending_transactions error");
        // we should now have one pending transaction since no block has been mined since this transaction was made
        assert_eq!(pending_transactions.len(), 1);
    }

    #[test]
    fn get_pending_transactions_has_no_tx_after_high_fee_transaction(){
        set_up();

        // // use get_default_mnenomic_words_2 so that you hae a fresh wallet not connected to the other wallet derived from get_default_mnenomic_words
        let mock_mnemonic = get_default_mnenomic_words_2();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        sleep_while_block_being_mined();

        let spent_transaction = new_master_account.spend_bitcoin(convert_float_to_satoshis(1.0), &get_base_address(), 10.0);
        test_result_type_is_not_err(spent_transaction);

        aw!(mine_a_block(&get_base_address()));
        sleep_while_block_being_mined();

        let pending_transactions = new_master_account.get_pending_transactions().unwrap();
        // // we should now have no pending transactions since a new block was mined
        assert_eq!(pending_transactions.len(), 0);
    }

    #[test]
    fn test_get_pending_spend_amount_reflects_unsettled_amount(){
        set_up();

        let mock_mnemonic = get_random_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        sleep_while_block_being_mined();
        new_master_account.sync_wallet().unwrap();

        let spent_transaction = new_master_account.spend_bitcoin(convert_float_to_satoshis(0.5), &get_base_address(), 1.0);
        test_result_type_is_not_err(spent_transaction);

        assert_eq!(new_master_account.get_pending_spend_amount().unwrap(), 100000141)
    }

    #[test]
    fn test_get_bitcoin_total_minus_transfers_to_children(){
        set_up();

        let mock_mnemonic = get_random_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic).unwrap();
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        // add one btc
        aw!(mine_a_block(&new_master_account.generate_new_address().unwrap().to_string()));
        sleep_while_block_being_mined();

        // set transfered amount to .4
        new_master_account.amount_transfered_to_children = convert_float_to_satoshis(0.4);
        let total_btc = new_master_account.get_bitcoin_total_minus_transfers_to_children().unwrap();

        assert_eq!(total_btc, convert_float_to_satoshis(0.6));
    }
}