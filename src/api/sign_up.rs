use actix_web::{post, web, HttpResponse, Responder, Error};
use crate::api::main_api::ApiSharedState;
use deadpool_postgres::{Client};

use crate::db::{actions, errors::MyError, models::user::User};

/// Add a new user to the database.
/// 
/// # Errors
/// If the data entered to create a user is invalid an error will be thrown.
pub async fn add_user(
    user: web::Json<User>,
    data: web::Data<ApiSharedState>,
) -> Result<User, Error> {
    let user_info: User = user.into_inner();

    let client: Client = data.db_pool.get().await.map_err(MyError::PoolError)?;

    let new_user = actions::user::add_user(&client, user_info).await?;

    Ok(new_user)
}

/// Create a new user.
#[post("/sign_up")]
pub async fn sign_up(
    user: web::Json<User>,
    data: web::Data<ApiSharedState>
) -> impl Responder {
    let add_user_result = add_user(user, data).await; //TODO handle error case
    // get request data 
    HttpResponse::Ok().json(add_user_result.unwrap())
}