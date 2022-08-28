use bdk::{miniscript, Wallet, KeychainKind, SyncOptions, SignOptions};
use bdk::database::MemoryDatabase;
use bdk::wallet::AddressIndex::New;
use bdk::wallet::AddressInfo;
use bdk::bitcoin::Address;
use bdk::bitcoin;
use bdk::FeeRate;
use bdk::keys::{DerivableKey, GeneratableKey, GeneratedKey, ExtendedKey, bip39::{Mnemonic, WordCount, Language}};
use bdk::template::Bip84;
use bdk::bitcoin::Network;
use std::str::FromStr;
use bdk::electrum_client::Client;
use bdk::blockchain::ElectrumBlockchain;
use bdk::blockchain::Blockchain;
use bdk::TransactionDetails;
use crate::helpers::convert_float_to_satoshis;
use std::{env, fmt};
use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words, get_default_mnenomic_words_2, mine_a_block, sleep_while_block_being_mined};
use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use bdk::{Error};
use bdk::bitcoin::Txid;
use futures::executor::block_on;
use tokio_test;
use std::{thread, time};


pub struct MasterAccount {
    pub bitcoin_amount: f64,
    pub all_addresses: Vec<AddressInfo>,
    pub account_addresses: Vec<AddressInfo>,
    pub wallet: Wallet<MemoryDatabase>,
    pub blockchain: Option<ElectrumBlockchain>,
    pub pending_transactions:Vec<Txid>,
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
    pub fn sync_wallet_with_electrum_server(&mut self, electrumUrl: Option<&str>){
        let electrumClientUrl = electrumUrl.unwrap_or("ssl://electrum.blockstream.info:60002");
        
        let client = Client::new(electrumClientUrl).unwrap(); 
        let blockchain = ElectrumBlockchain::from(client);
        
        self.wallet.sync(&blockchain, SyncOptions::default()).unwrap();
        self.set_blockchain(blockchain);
    }

    pub fn set_blockchain(&mut self, blockchain: ElectrumBlockchain){
        self.blockchain = Some(blockchain);
    }

    pub fn spend_bitcoin(&mut self, amount: f64, address: &str, sat_per_vb: f32 ) -> Result<&'static str, &'static str>{
        // TODO 
        // actually create, sign and broatcast a transaction here, will need to know where to send it to.

        let receiving_address = Address::from_str(address).unwrap();
        let mut tx_builder = self.wallet.build_tx();
        tx_builder
            .add_recipient(receiving_address.script_pubkey(), convert_float_to_satoshis(amount))
            .enable_rbf().fee_rate(FeeRate::from_sat_per_vb(sat_per_vb));

        let (mut psbt, tx_details) = tx_builder.finish().unwrap();

        println!("tx_details is {:?}", tx_details);
    
        let finalized = self.wallet.sign(&mut psbt, SignOptions::default()).unwrap();
        
        // now broadcast it 
        let raw_transaction = psbt.extract_tx();
        let txid = raw_transaction.txid();
        println!("the txid {}", txid);
        let myBlockchain = self.blockchain.as_ref();
        
        myBlockchain.unwrap().broadcast(&raw_transaction);
        self.wallet.sync(myBlockchain.unwrap(), SyncOptions::default());

        // now we have a pending transaction, so add it to the list of pending_transactions
        self.pending_transactions.push(txid);

        self.bitcoin_amount = self.bitcoin_amount - amount;
        println!("you have spent {} bitcoin, you now have {} remaining", amount, self.bitcoin_amount);
        Ok("Success")
    }

    pub fn generate_new_address(&mut self) -> Address {        
        let my_new_address = self.wallet.get_address(New).unwrap();
        let copied_address = my_new_address.clone();
        self.all_addresses.push(my_new_address);
        copied_address 
    }

    pub fn get_bitcoin_total(&self)-> Result<u64, Error> { 
       self.wallet.get_balance()
    }

    pub fn get_pending_transactions(&mut self) -> &Vec<Txid>{
        // resync the blockchain to the wallet again to get the latest data.
        self.wallet.sync(self.blockchain.as_ref().unwrap(), SyncOptions::default());

        // for each pending transaction go check if it is still pending
        let mut transactions_that_are_no_longer_pending: Vec<Txid> = vec![];
        for txId in &self.pending_transactions{
            let mut myTransaction  = self.wallet.get_tx(txId, false).unwrap().unwrap();
            
            if myTransaction.confirmation_time != None{
                // remove it from the list if it has been confirmed
                transactions_that_are_no_longer_pending.push(myTransaction.txid.clone());
            }
            println!("txId {} this is my transaction {:?}", txId, myTransaction);
        }
        // filter out the pending transactions, removing the ones that have been confirmed
        for item in transactions_that_are_no_longer_pending{
            let index = self.pending_transactions.iter().position(|txId| txId.to_string() == item.to_string());
            self.pending_transactions.remove(index.unwrap());
        }
        self.pending_transactions.as_ref()
    }
}


pub mod test {
    use super::*;

    #[test]
    fn master_account_initialized_with_no_bitcoin(){
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);

        assert_eq!(new_master_account.bitcoin_amount, 0.0)
    }
    #[test]
    fn spend_bitcoin_returns_success_and_reduces_bitcoin_amount(){
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);


        new_master_account.bitcoin_amount = 2.0;
        let response = new_master_account.spend_bitcoin(1.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax", 1.0).unwrap();
        assert_eq!(new_master_account.bitcoin_amount, 1.0);
        assert_eq!(response, "Success")
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

        new_master_account.spend_bitcoin(1.0, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw", 1.0);
        let pending_transactions = new_master_account.get_pending_transactions();
        // we should now have one pending transaction since no block has been mined since this transaction was made
        assert_eq!(pending_transactions.len(), 1);
    }

    // used to handle async await functions
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
      }

    #[test]
    fn get_pending_transactions_has_no_tx_after_high_fee_transaction(){
        // // use get_default_mnenomic_words_2 so that you hae a fresh wallet not connected to the other wallet derived from get_default_mnenomic_words
        let mock_mnemonic = get_default_mnenomic_words_2();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&mut new_master_account);
        
        new_master_account.spend_bitcoin(1.0, "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw", 10.0);
        aw!(mine_a_block());
        sleep_while_block_being_mined();

        let pending_transactions = new_master_account.get_pending_transactions();
        // // we should now have no pending transactions since a new block was mined
        assert_eq!(pending_transactions.len(), 0);
    }
}