
pub mod head_of_the_house;
pub mod child;
pub mod permissions;
pub mod account;

use head_of_the_house::HeadOfTheHouse;
use permissions::BitcoinPermissions;
fn main() {
    let mut master = HeadOfTheHouse::new();
    // master.add_child(2, String::from("my new child"),  vec![BitcoinPermissions::Send, BitcoinPermissions::Receive] );
    
    // println!("Hello, world! {:?}", master.children.get(0));
}
