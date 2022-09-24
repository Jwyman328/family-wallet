use std::{thread, time};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::head_of_the_house::HeadOfTheHouse;
use crate::api::main_api::{AppState, AppStateWithCounter};

#[get("/hello")]
pub async fn hello() -> impl Responder {
    // TODO how do we get the master wallet into this function.
    HttpResponse::Ok().body("Hello world!")
}

#[get("/")]
pub async fn with_data(data: web::Data<AppStateWithCounter>) -> impl Responder {
    // let app_name = &data.app_name; // <- get app_name
    let app_name = "hi";
    let string_formated = format!("Hello {app_name}!");
    HttpResponse::Ok().body(string_formated)
}

#[get("/count")]
pub async fn count_request(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut count = data.counter.lock().unwrap(); // <- get app_name
    *count += 1;
    let string_formated = format!("count is {count}!");
    HttpResponse::Ok().body(string_formated)
}

#[get("/count_and_wait")]
pub async fn count_request_and_wait(data: web::Data<AppStateWithCounter>) -> impl Responder {
    
    let mut count = data.counter.lock().unwrap(); // <- get app_name
    *count += 1;
    std::mem::drop(count); // don't block other requests.
    
    thread::sleep(time::Duration::from_millis(5000));
    let count = data.counter.lock().unwrap(); // get freshcount
    let string_formated = format!("count is {count}!");
    HttpResponse::Ok().body(string_formated)
}

/// Try out the master mutex being passed in
#[get("/get_balance_from_wallet")]
pub async fn get_balance_from_wallet(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut master = data.master.lock().unwrap();
    let my_current_balance =  master.get_account_balance_utxo_amount_plus_transfer_balance(1).unwrap();
    // print a new address just to for testing purposes 
    let my_next_addresss = master.get_new_address(1).unwrap();
    HttpResponse::Ok().body(format!("my balance {my_current_balance}, my next address {my_next_addresss}"))
}


pub async fn manual_hello(mut master:HeadOfTheHouse) -> impl Responder {
    let total = master.get_account_balance_utxo_amount_plus_transfer_balance(1).unwrap();
    let string_total = total.to_string();
    HttpResponse::Ok().body(string_total)
}

pub async fn manual_hello_without_input() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}