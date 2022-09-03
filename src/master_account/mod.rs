use bdk::{miniscript, Wallet, KeychainKind, SyncOptions, SignOptions};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bdk::wallet::AddressInfo;
use bdk::bitcoin::{Address, Network, Txid};
use bdk::FeeRate;
use bdk::keys::{DerivableKey, GeneratableKey, GeneratedKey, ExtendedKey, bip39::{Mnemonic, WordCount, Language}};
use bdk::template::Bip84;
use std::str::FromStr;
use bdk::electrum_client::Client;
use bdk::blockchain::ElectrumBlockchain;
use bdk::blockchain::Blockchain;
use bdk::TransactionDetails;
use crate::helpers::{convert_float_to_satoshis, convert_satoshis_to_float};
use bdk::{Error};



pub struct MasterAccount {
    pub bitcoin_amount: f64,
    pub all_addresses: Vec<AddressInfo>,
    pub account_addresses: Vec<AddressInfo>,
    pub wallet: Wallet<MemoryDatabase>,
    pub blockchain: Option<ElectrumBlockchain>,
    pub pending_transactions:Vec<TransactionDetails>,
}

impl MasterAccount {
    pub fn new(words:Option<String>) -> MasterAccount{
        let wallet = MasterAccount::generate_wallet(words);
        MasterAccount {
            bitcoin_amount: 0.0,
            all_addresses: Vec::new(),
            account_addresses: Vec::new(),
            wallet: wallet,
            blockchain: None,
            pending_transactions: vec![]
        }
    }

    pub fn generate_wallet(words:Option<String>)-> Wallet<MemoryDatabase> {
        // if provided words, then use them to generate a wallet,
        // if not then generate your own randomly
        let mnemonic_words = match words {
            Some(w) => w,
            _ => {
                let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
                // Convert mnemonic to string
                let mnemonic_words = mnemonic.to_string();
                mnemonic_words
            },
        };

        println!("my mnemonic words {}", mnemonic_words);

        let network = Network::Regtest; // Or this can be Network::Bitcoin, Network::Signet or Network::Testnet

        // Parse a mnemonic
        let mnemonic  = Mnemonic::parse(&mnemonic_words).unwrap();
        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).unwrap();

        let wallet = Wallet::new(
            Bip84(xprv, KeychainKind::External),
            Some(Bip84(xprv, KeychainKind::Internal)),
            network,
            MemoryDatabase::default(),
        )
        .unwrap();

        wallet
    }

    /// This function connects to a currently running electrum server 
    /// and returns the ElectrumBlockchain struct. 
    /// this can be an running electrum server, by default we will connect to blockstreams
    /// but it could be a local server as well like a regtest one created from nigiri at 127.0.0.1:50000
    pub fn sync_wallet_with_electrum_server(&mut self, electrum_url: Option<&str>){
        let electrum_client_url = electrum_url.unwrap_or("ssl://electrum.blockstream.info:60002");
        
        let client = Client::new(electrum_client_url).unwrap(); 
        let blockchain = ElectrumBlockchain::from(client);
        
        self.wallet.sync(&blockchain, SyncOptions::default()).unwrap();
        self.set_blockchain(blockchain);
    }

    pub fn set_blockchain(&mut self, blockchain: ElectrumBlockchain){
        self.blockchain = Some(blockchain);
    }

    pub fn spend_bitcoin(&mut self, amount: f64, address: &str, sat_per_vb: f32 ) -> Result<TransactionDetails, &'static str>{
        // make sure our wallet is up to date before we make a spend.
        self.sync_wallet();

        let receiving_address = Address::from_str(address).unwrap();
        let mut tx_builder = self.wallet.build_tx();
        tx_builder
            .add_recipient(receiving_address.script_pubkey(), convert_float_to_satoshis(amount))
            .enable_rbf().fee_rate(FeeRate::from_sat_per_vb(sat_per_vb));

        let (mut psbt, tx_details) = tx_builder.finish().unwrap();

        println!("tx_details is {:?}", tx_details);
    
        self.wallet.sign(&mut psbt, SignOptions::default()).unwrap();
        
        // now broadcast it 
        let raw_transaction = psbt.extract_tx();
        let txid = raw_transaction.txid();
        println!("the txid {}", txid);
        let my_blockchain = self.blockchain.as_ref();
        
        my_blockchain.unwrap().broadcast(&raw_transaction);
        self.sync_wallet();

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

        self.bitcoin_amount = self.bitcoin_amount - amount;
        println!("you have spent {} bitcoin, you now have {} remaining", amount, self.bitcoin_amount);

        Ok(copied_transaction)
    }

    pub fn generate_new_address(&mut self) -> Address {        
        let my_new_address = self.wallet.get_address(New).unwrap();
        let copied_address = my_new_address.clone();
        self.all_addresses.push(my_new_address);
        copied_address 
    }

    pub fn get_pending_spend_amount(&mut self)-> f64{ 
      let pending_transactions = self.get_pending_transactions();
      
      let mut pending_spend_amount = 0;
      for transaction in pending_transactions{
        pending_spend_amount += transaction.sent;
        pending_spend_amount += transaction.fee.unwrap();
      }
      convert_satoshis_to_float(pending_spend_amount) 
    }

    pub fn get_bitcoin_total(&self)-> Result<u64, Error> { 
        self.wallet.get_balance()
     }

    pub fn get_pending_transactions(&mut self) -> &Vec<TransactionDetails>{
        // resync the blockchain to the wallet again to get the latest data.
        self.sync_wallet();

        // for each pending transaction go check if it is still pending
        let mut transactions_that_are_no_longer_pending: Vec<Txid> = vec![];
        for transaction_detail in &self.pending_transactions{
            let my_transaction  = self.wallet.get_tx(&transaction_detail.txid, false).unwrap().unwrap();
            
            if my_transaction.confirmation_time != None{
                // remove it from the list if it has been confirmed
                transactions_that_are_no_longer_pending.push(my_transaction.txid.clone());
            }
            println!("txId {} this is my transaction {:?}", transaction_detail.txid, my_transaction);
        }
        // filter out the pending transactions, removing the ones that have been confirmed
        for item in transactions_that_are_no_longer_pending{
            let index = self.pending_transactions.iter().position(|tx_detail| tx_detail.txid.to_string() == item.to_string());
            self.pending_transactions.remove(index.unwrap());
        }
        self.pending_transactions.as_ref()
    }

    pub fn sync_wallet(&self){
        self.wallet.sync(self.blockchain.as_ref().unwrap(), SyncOptions::default());
    }
}

#[cfg(test)]
pub mod test {
    use crate::testing_helpers::{get_random_mnenomic_words, test_result_type_is_not_err};

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
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);

        assert_eq!(new_master_account.bitcoin_amount, 0.0)
    }
    #[test]
    #[ignore] //fix test since we have new structure.
    fn spend_bitcoin_returns_success_and_reduces_bitcoin_amount(){
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);


        new_master_account.bitcoin_amount = 2.0;
        let response = new_master_account.spend_bitcoin(1.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax", 1.0);
        test_result_type_is_not_err(response);
        assert_eq!(new_master_account.bitcoin_amount, 1.0);
        // assert_eq!(response, "Success")
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
        // // use get_default_mnenomic_words_2 so that you hae a fresh wallet not connected to the other wallet derived from get_default_mnenomic_words
        let mock_mnemonic = get_default_mnenomic_words_2();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);

        let spent_transaction = new_master_account.spend_bitcoin(1.0, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw", 1.0);
        test_result_type_is_not_err(spent_transaction);

        let pending_transactions = new_master_account.get_pending_transactions();
        // we should now have one pending transaction since no block has been mined since this transaction was made
        assert_eq!(pending_transactions.len(), 1);
    }

    #[test]
    fn get_pending_transactions_has_no_tx_after_high_fee_transaction(){
        // // use get_default_mnenomic_words_2 so that you hae a fresh wallet not connected to the other wallet derived from get_default_mnenomic_words
        let mock_mnemonic = get_default_mnenomic_words_2();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        
        let spent_transaction = new_master_account.spend_bitcoin(1.0, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw", 10.0);
        test_result_type_is_not_err(spent_transaction);

        aw!(mine_a_block("bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw"));
        sleep_while_block_being_mined();

        let pending_transactions = new_master_account.get_pending_transactions();
        // // we should now have no pending transactions since a new block was mined
        assert_eq!(pending_transactions.len(), 0);
    }

    #[test]
    fn test_get_pending_spend_amount_reflects_unsettled_amount(){
        let mock_mnemonic = get_random_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        aw!(mine_a_block(&new_master_account.generate_new_address().to_string()));
        sleep_while_block_being_mined();

        let spent_transaction = new_master_account.spend_bitcoin(0.5, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw", 1.0);
        test_result_type_is_not_err(spent_transaction);

        assert_eq!(new_master_account.get_pending_spend_amount(), 100000141.0)
    }
}