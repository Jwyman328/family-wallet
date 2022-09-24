
pub mod head_of_the_house;
pub mod child;
pub mod permissions;
pub mod account;
pub mod master_account;
pub mod children;
pub mod helpers;
pub mod testing_helpers;
pub mod env_variables;
pub mod custom_errors;
pub mod api;

use head_of_the_house::HeadOfTheHouse;
// use permissions::BitcoinPermissions;
use crate::testing_helpers::{mine_a_block, get_random_mnenomic_words, get_regtest_rpc};
use futures::executor::block_on;
use tokio;
use env_variables::{set_env_variables};
use api::main_api::{main_api};
use children::Children;

fn main() {
    set_env_variables(Some(true));

    // TODO what do we needd to set up.
    // an electrum server
    // master account etc
    let mut default_children = Children::new();
    let random_words = get_random_mnenomic_words();
    let mut master = HeadOfTheHouse::new(&mut default_children, random_words).unwrap();
    // let receiving_address = master.get_new_address(1).unwrap();
    // block_on( mine_a_block(&receiving_address.to_string()));

    // use the nigiri electrum regtest server for now and mine a block to an address 
    // dont forget the nigiri electrum server must be running.
    let nigiri_electrum_server = Some("127.0.0.1:50000");
    master.master_account.sync_wallet_with_electrum_server(nigiri_electrum_server); // sync with blockstream by default
    
    main_api(master);

    // println!("Hello, world! {:?}", master.children.get(0));
}
