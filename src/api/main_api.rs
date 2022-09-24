use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use crate::api::get_balance::{hello, manual_hello, manual_hello_without_input, with_data, count_request, count_request_and_wait, get_balance_from_wallet};
use crate::HeadOfTheHouse;
use std::sync::{Mutex, Arc};
pub struct AppState {
    pub app_name: String,
    new_counter:AppStateWithCounter
}

pub struct AppStateWithCounter {
    pub counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
    pub master: Arc<Mutex<HeadOfTheHouse>>
}

// I want each thread to increment the count.
// one way I could do this is by locking a mutex, and adding to it, then unlocking it.


#[actix_web::main]
pub async fn main_api(mut master:HeadOfTheHouse) -> std::io::Result<()> {
    let master_mutex = Arc::new(Mutex::new(master)); // allows sharing across multiple threads, but is blocking when locked.
    // other option is to start one up on each request
    // or start up like ten on app start and then allocate them with each request.


    let newCounter = Mutex::new(0);
    let webData = web::Data::new(
        AppStateWithCounter {
        counter: newCounter,
        master: Arc::clone(&master_mutex),
    });

    HttpServer::new(move|| {
        App::new().app_data(webData.clone())
        .service(hello)
        .service(with_data)
        .service(count_request)
        .service(count_request_and_wait)
        .service(get_balance_from_wallet)
        .route("/hey", web::get().to(manual_hello_without_input))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await

}