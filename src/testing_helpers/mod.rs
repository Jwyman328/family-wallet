use crate::master_account::MasterAccount;
use std::collections::HashMap;
use std::{thread, time};

/// set tests up to use our regtest nigiri electrum server
/// hosted at 127.0.0.1:50000
pub fn attach_wallet_to_regtest_electrum_server(master_account: &mut MasterAccount ){
    master_account.sync_wallet_with_electrum_server(Some("127.0.0.1:50000"));
}

pub fn get_default_mnenomic_words()-> Option<String>{
    return Some(String::from("jelly crash boy whisper mouse ecology tuna soccer memory million news short"));
}

pub fn get_default_mnenomic_words_2()-> Option<String>{
    return Some(String::from("talk again shop lizard found all elite argue ride misery drama street"));
}

/// Add more bitcoin to an address, this will automatically mine a block.
/// on the esplora regtest that is running on localhost 3000.
pub async fn mine_a_block()-> reqwest::Response{
    let mut map = HashMap::new();
    map.insert("address", "bcrt1q2ltw5646zcdxcj7hvv47mklqy8la6ta83p6egw");

    let client = reqwest::Client::new();
    let res = client.post("http://localhost:3000/faucet")
        .json(&map)
        .send()
        .await.unwrap();
    res
}

pub fn sleep_while_block_being_mined(){
    let ten_millis = time::Duration::from_millis(4000);

    thread::sleep(ten_millis);
}