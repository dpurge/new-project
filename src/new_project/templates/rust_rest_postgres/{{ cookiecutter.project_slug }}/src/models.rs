use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}
