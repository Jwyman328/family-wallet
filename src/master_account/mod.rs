use bdk::{miniscript, Wallet, KeychainKind, SyncOptions};
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
use crate::helpers::convert_float_to_satoshis;
use std::env;
use crate::testing_helpers::{attach_wallet_to_regtest_electrum_server, get_default_mnenomic_words};

#[derive(Debug)]

pub struct MasterAccount {
    pub bitcoin_amount: f64,
    pub all_addresses: Vec<AddressInfo>,
    pub account_addresses: Vec<AddressInfo>,
    pub wallet: Wallet<MemoryDatabase>,
}

impl MasterAccount {
    pub fn new(words:Option<String>) -> MasterAccount{
        let wallet = MasterAccount::generate_wallet(words);
        MasterAccount {
            bitcoin_amount: 0.0,
            all_addresses: Vec::new(),
            account_addresses: Vec::new(),
            wallet: wallet
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
    pub fn sync_wallet_with_electrum_server(&self, electrumUrl: Option<&str>){
        let electrumClientUrl = electrumUrl.unwrap_or("ssl://electrum.blockstream.info:60002");
        println!("i hope so {}",electrumClientUrl);
        let client = Client::new(electrumClientUrl).unwrap(); 
        let blockchain = ElectrumBlockchain::from(client);

        self.wallet.sync(&blockchain, SyncOptions::default()).unwrap();
    }

    pub fn spend_bitcoin(&mut self, amount: f64, address: &str ) -> Result<&'static str, &'static str>{
        // TODO 
        // actually create, sign and broatcast a transaction here, will need to know where to send it to.

        let receiving_address = Address::from_str(address).unwrap();
        let mut tx_builder = self.wallet.build_tx();
        tx_builder
            .add_recipient(receiving_address.script_pubkey(), convert_float_to_satoshis(amount))
            .enable_rbf().fee_rate(FeeRate::from_sat_per_vb(1.0));

        let (mut psbt, tx_details) = tx_builder.finish().unwrap();

        println!("tx_details is {:?}", tx_details);

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
}


pub mod test {
    use super::*;

    #[test]
    fn master_account_initialized_with_no_bitcoin(){
        let mock_mnemonic = get_default_mnenomic_words();

        let new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&new_master_account);

        assert_eq!(new_master_account.bitcoin_amount, 0.0)
    }
    #[test]
    fn spend_bitcoin_returns_success_and_reduces_bitcoin_amount(){
        let mock_mnemonic = get_default_mnenomic_words();

        let mut new_master_account = MasterAccount::new(mock_mnemonic);
        attach_wallet_to_regtest_electrum_server(&new_master_account);


        new_master_account.bitcoin_amount = 2.0;
        let response = new_master_account.spend_bitcoin(1.0, "tb1qapswup3gzwzmwqp9sk7s5zvm3v9n00x6whp7ax").unwrap();
        assert_eq!(new_master_account.bitcoin_amount, 1.0);
        assert_eq!(response, "Success")
    }
}