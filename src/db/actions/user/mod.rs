use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::db::{errors::MyError, models::user::User};

/// Add a user to the postgres database with an sql command.
/// 
/// Take the user_info and insert it into the database. After it is inserted
/// query for the data to make sure it was added correctly and return User.
/// 
/// # Errors
/// If the user data is not added correctly and therefore can not be retrieved with
/// a query, return a MyError::NotFound.
/// 
/// TODO handle the unwraps correctly by returning an error instead of panicing.
pub async fn add_user(client: &Client, user_info: User) -> Result<User, MyError> {
    let _stmt = include_str!("../../sql/add_user.sql");
    let _stmt = _stmt.replace("$table_fields", &User::sql_table_fields());
    let stmt = client.prepare(&_stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &user_info.email,
                &user_info.first_name,
                &user_info.last_name,
                &user_info.username,
            ],
        )
        .await?
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>()
        .pop()
        .ok_or(MyError::NotFound) // more applicable for SELECTs
}