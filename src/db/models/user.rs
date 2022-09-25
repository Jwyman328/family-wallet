use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

/// A User struct with auto generated impl functions 
/// due to the derive macros like PostgresMapper.
/// 
/// # Examples 
/// you can see an example of the auto generated impl function like 
/// `from_row_ref` in the models/user.rs. Which will take postgres row data
/// and convert it into the User struct.
#[derive(Deserialize, PostgresMapper, Serialize)]
#[pg_mapper(table = "users")] // singular 'user' is a keyword..
pub struct User {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}