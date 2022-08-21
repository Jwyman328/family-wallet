use crate::master_account::MasterAccount;

/// set tests up to use our regtest nigiri electrum server
/// hosted at 127.0.0.1:50000
pub fn attach_wallet_to_regtest_electrum_server(master_account: &MasterAccount ){
    master_account.sync_wallet_with_electrum_server(Some("127.0.0.1:50000"));
}

pub fn get_default_mnenomic_words()-> Option<String>{
    return Some(String::from("jelly crash boy whisper mouse ecology tuna soccer memory million news short"));
}