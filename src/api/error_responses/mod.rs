use serde::{ Serialize};

#[derive(Serialize)]
pub struct BadRequestResponse {
    pub msg: String,
}

