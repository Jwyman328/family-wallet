use crate::master_account::MasterAccount;
use std::collections::HashMap;
use std::{thread, time, env};
use bdk::bitcoin::psbt::PartiallySignedTransaction;
use bdk::keys::{ GeneratableKey, GeneratedKey, bip39::{Mnemonic, WordCount, Language}};
use bdk::{miniscript, TransactionDetails};
use bdk::bitcoin::Address;
use std::str::FromStr;
use bdk::Wallet;
use bdk::database::MemoryDatabase;
use crate::helpers::convert_float_to_satoshis;
use bdk::FeeRate;

use crate::env_variables::set_env_variables;

/// Set tests up to use our regtest nigiri electrum server
/// hosted at 127.0.0.1:50000
/// 
/// # Panics
/// If there is no electrum_server environment variable panic the app.
pub fn attach_wallet_to_regtest_electrum_server(master_account: &mut MasterAccount ){
    let default_electrum_server = env::var("electrum_server").expect("error setting default_electrum_server");
    master_account.sync_wallet_with_electrum_server(Some(&default_electrum_server));
}

/// Get default mnenomic words in order to have a consistant wallet across some tests.
pub fn get_default_mnenomic_words()-> Option<String>{
    return Some(String::from("jelly crash boy whisper mouse ecology tuna soccer memory million news short"));
}

/// Get a different set of mnenomic words in order to have a different consistant wallet across some tests.
pub fn get_default_mnenomic_words_2()-> Option<String>{
    return Some(String::from("talk again shop lizard found all elite argue ride misery drama street"));
}

/// Get a random set of mnenomic words in order to generate a brand new wallet.
pub fn get_random_mnenomic_words()-> Option<String>{
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).expect("error generating mnemonic");
    let mnemonic_words = mnemonic.to_string();
    Some(mnemonic_words)
}

/// Automatically mine a block which adds bitcoin to a `receiving_address`.
/// 
/// This will happen on the local esplora regtest blockchain that is running on localhost 3000.
pub async fn mine_a_block(receiving_address: &str)-> reqwest::Response{
    let mut map = HashMap::new();
    map.insert("address", receiving_address);

    let client = reqwest::Client::new();
    let regtest_rpc = get_regtest_rpc();
    let regtest_generate_block_url = regtest_rpc + "/faucet";
    let res = client.post(regtest_generate_block_url)
        .json(&map)
        .send()
        .await.expect("error attempting to mine_a_block");
    res
}

/// A block being mined takes a few seconds, therefore we should pause the test
/// while this is happening in order to have an updated state after a block is mined.
pub fn sleep_while_block_being_mined(){
    let ten_millis = time::Duration::from_millis(6000);

    thread::sleep(ten_millis);
}

/// A function which will test that a result type is not an Error, if it is 
/// it will fail the test.
pub fn test_result_type_is_not_err<T, E>(result:Result<T,E>){
    match result {
        Ok(_) => assert_eq!(true, true), // is an okay type so pass test
        _ => assert_eq!(true, false) // we got an error so fail the test
    }
}

/// A global set up function for all tests which will set up all the neccesary environment variables.
pub fn set_up(){
    set_env_variables();
}

/// Get the `test_address` environment variables
/// 
/// # Panics
/// If the environment variable test_address does not exist.
pub fn get_base_address()-> String{
    env::var("test_address").expect("Error getting test_address env var")
}

/// Get the `regtest_rpc` environment variables
/// 
/// # Panics
/// If the environment variable regtest_rpc does not exist.
pub fn get_regtest_rpc()-> String {
    env::var("regtest_rpc").expect("error getting regtest_rpc env var")
}

/// Build a psbt for an amount.
/// 
/// # Panics
/// If the environment variable `test_address` is an invalid address.
pub fn build_mock_transaction(wallet:&Wallet<MemoryDatabase>, mock_amount:u64)->(PartiallySignedTransaction, TransactionDetails){
    let test_address = get_base_address();
    let receiving_address = Address::from_str(&test_address).expect("Error in build_mock_transaction receiving_address");

    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(receiving_address.script_pubkey(), mock_amount)
        .enable_rbf().fee_rate(FeeRate::from_sat_per_vb(1.0));

    let (psbt, tx_details) = tx_builder.finish().expect("error building mock_transaction");
    (psbt, tx_details)
}