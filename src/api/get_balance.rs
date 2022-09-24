use actix_web::{get, web, HttpResponse, Responder};
use crate::api::main_api::WalletIniliazer;


/// Get the current wallet balance
#[get("/get_balance_from_wallet")]
pub async fn get_balance_from_wallet(data: web::Data<WalletIniliazer>) -> impl Responder {
    let mut head_of_house = data.get_head_of_house().unwrap(); // TODO remove unwrap

    let my_current_balance =  head_of_house.get_account_balance_utxo_amount_plus_transfer_balance(1).unwrap();
    // print a new address just to for testing purposes 
    let my_next_addresss = head_of_house.get_new_address(1).unwrap();
    HttpResponse::Ok().body(format!("my balance {my_current_balance}, my next address {my_next_addresss}"))
}