use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;

use crate::{repository, Config};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn router(_config: &Config, pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/meta/entities", get(list_entities).post(create_entity))
        .with_state(AppState { pool })
}

#[derive(Debug, Deserialize)]
pub struct CreateEntity {
    pub id: String,
    pub name: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

async fn list_entities(
    State(state): State<AppState>,
) -> Result<Json<Vec<repository::Entity>>, (axum::http::StatusCode, Json<ErrorBody>)> {
    repository::list_entities(&state.pool)
        .await
        .map(Json)
        .map_err(internal_error)
}

async fn create_entity(
    State(state): State<AppState>,
    Json(input): Json<CreateEntity>,
) -> Result<
    (axum::http::StatusCode, Json<repository::Entity>),
    (axum::http::StatusCode, Json<ErrorBody>),
> {
    repository::create_entity(&state.pool, &input.id, &input.name, &input.label)
        .await
        .map(|entity| (axum::http::StatusCode::CREATED, Json(entity)))
        .map_err(internal_error)
}

fn internal_error(error: anyhow::Error) -> (axum::http::StatusCode, Json<ErrorBody>) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            code: "internal_error",
            message: error.to_string(),
        }),
    )
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "status": "ok" }))
}

async fn version() -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "name": "logholizon-core", "version": env!("CARGO_PKG_VERSION") }))
}
