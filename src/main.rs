
pub mod head_of_the_house;
pub mod child;
pub mod permissions;
pub mod account;
pub mod master_account;
pub mod children;
pub mod helpers;
pub mod testing_helpers;

use head_of_the_house::HeadOfTheHouse;
use permissions::BitcoinPermissions;
use crate::testing_helpers::mine_a_block;
use futures::executor::block_on;
use tokio;

#[tokio::main]
async fn main() {
    block_on( mine_a_block());
    // let mut master = HeadOfTheHouse::new();
    // master.master_account.sync_wallet_with_electrum_server(None) // sync with blockstream by default
    // master.add_child(2, String::from("my new child"),  vec![BitcoinPermissions::Send, BitcoinPermissions::Receive] );
    
    // println!("Hello, world! {:?}", master.children.get(0));
}
