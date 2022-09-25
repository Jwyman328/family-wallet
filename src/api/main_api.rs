use actix_web::{web, App, HttpServer};
use crate::api::get_balance::{ get_balance_from_wallet};
use crate::api::sign_up::sign_up;
use crate::HeadOfTheHouse;
use crate::custom_errors::{ AccountError};
use std::env;
use crate::children::Children;
use ::config::Config;
use dotenv::dotenv;
use tokio_postgres::NoTls;
use deadpool_postgres::{Pool, Client, Manager};
use crate::db::config::ExampleConfig;

/// A struct that may contain seed words used to initalize a bitcoin wallet.
pub struct WalletIniliazer {
   pub words: Option<String>,
   pub db_pool: Pool
}

impl WalletIniliazer {
    /// Initialize and get the head of the house.
    /// 
    /// # Errors
    /// Same as `initiate_wallet`.
    pub fn get_head_of_house(&self)-> Result<HeadOfTheHouse, AccountError>{
        initiate_wallet(self.words.clone())
    }
}


/// Initalize a wallet through the `HeadOfTheHouse` and sync it with an electrum server.
/// 
/// # Errors
/// If the `HeadOfTheHouse` can not initiate with the passed in mnemonic.
/// 
/// # Panics
/// If the electrum server env var is not initiated then we have an app wide issue, and should panic.
pub fn initiate_wallet(mnemonic:Option<String>)-> Result<HeadOfTheHouse, AccountError> {
    let mut default_children = Children::new();
    let mut head_of_the_house = HeadOfTheHouse::new(&mut default_children, mnemonic)?;
    let electrum_server = env::var("electrum_server").unwrap();
    head_of_the_house.master_account.sync_wallet_with_electrum_server(Some(&electrum_server));
    Ok(head_of_the_house)
}

/// The main api server function, here we generate our shared state `api_shared_state`
/// and pass it to each endpoint function.
/// 
/// # Panics 
/// If the env variables url_location or location_port do not exist, the server will not be able to mount.
#[actix_web::main]
pub async fn main_api(mnemonic:Option<String>) -> std::io::Result<()> {
    dotenv().ok();

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();

    let config: ExampleConfig = config_.try_deserialize().unwrap();

    let pool = config.pg.create_pool(None, NoTls).unwrap();


    //
    let wallet_initializer =  WalletIniliazer {
        words: mnemonic,
        db_pool: pool
    };
    
    let api_shared_state = web::Data::new(
        wallet_initializer,
        
    );

    let url_location = env::var("url_location").unwrap();
    let location_port = env::var("location_port").unwrap().parse::<u16>().unwrap();

    HttpServer::new(move|| {
        App::new().app_data(api_shared_state.clone())
        .service(get_balance_from_wallet)
        .service(sign_up)
    })
    .bind((url_location, location_port))?
    .run()
    .await
}