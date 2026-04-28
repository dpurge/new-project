use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    error::AppError,
    models::{CreateTodo, Todo},
    state::AppState,
};

pub async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed FROM todos ORDER BY id DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(todos))
}

pub async fn get_todo(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed FROM todos WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("todo {id} was not found")))?;

    Ok(Json(todo))
}

pub async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = payload.title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest(
            "title must not be empty".to_owned(),
        ));
    }

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title) VALUES ($1) RETURNING id, title, completed",
    )
    .bind(title)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(todo)))
}
