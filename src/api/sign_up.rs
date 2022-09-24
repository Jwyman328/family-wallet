use actix_web::{post, web, HttpResponse, Responder};
use crate::api::main_api::WalletIniliazer;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupInfo {
    pub username: String,
    pub password: String
}

/// Get the current wallet balance
#[post("/sign_up")]
pub async fn sign_up(info: web::Json<SignupInfo>, data: web::Data<WalletIniliazer>) -> impl Responder {
    // get request data 
    HttpResponse::Ok().body(format!("what I receives was {}", info.username))
}