use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    repository::{self, CreateDocument, UpdateDocument},
    Config,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn router(_config: &Config, pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/meta/entities", get(list_entities).post(create_entity))
        .route("/v1/meta/entities/{id}", get(get_entity))
        .route("/v1/meta/entities/{id}/workflow", get(get_workflow))
        .route("/v1/documents", get(list_documents).post(create_document))
        .route(
            "/v1/documents/{id}",
            get(get_document)
                .put(update_document)
                .delete(delete_document),
        )
        .route("/v1/documents/{id}/audit", get(list_document_audit))
        .route(
            "/v1/documents/{id}/transition",
            axum::routing::post(transition_document),
        )
        .route("/v1/dashboard/counts", get(dashboard_counts))
        .with_state(AppState { pool })
}

#[derive(Debug, Deserialize)]
pub struct CreateEntity {
    pub id: String,
    pub name: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub id: String,
    pub entity_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub entity_id: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

async fn list_entities(
    State(state): State<AppState>,
) -> Result<Json<Vec<repository::Entity>>, AppError> {
    repository::list_entities(&state.pool)
        .await
        .map(Json)
        .map_err(AppError::from)
}

async fn create_entity(
    State(state): State<AppState>,
    Json(input): Json<CreateEntity>,
) -> Result<(StatusCode, Json<repository::Entity>), AppError> {
    repository::create_entity(&state.pool, &input.id, &input.name, &input.label)
        .await
        .map(|entity| (StatusCode::CREATED, Json(entity)))
        .map_err(map_db_error)
}

async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::EntityDetail>, AppError> {
    repository::get_entity_detail(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::WorkflowDefinition>, AppError> {
    repository::get_workflow(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub action: String,
}

async fn transition_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<TransitionRequest>,
) -> Result<Json<repository::Document>, AppError> {
    if input.action.trim().is_empty() {
        return Err(AppError::BadRequest("action is required".into()));
    }
    repository::transition_document(&state.pool, &id, &input.action)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub entity_id: String,
}

async fn dashboard_counts(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<Vec<repository::StatusCount>>, AppError> {
    if query.entity_id.trim().is_empty() {
        return Err(AppError::BadRequest("entity_id is required".into()));
    }
    repository::count_documents_by_status(&state.pool, &query.entity_id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<repository::DocumentList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::list_documents(&state.pool, &query.entity_id, query.limit, query.offset)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn create_document(
    State(state): State<AppState>,
    Json(input): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<repository::Document>), AppError> {
    repository::create_document(
        &state.pool,
        &input.id,
        &input.entity_id,
        &CreateDocument {
            payload: input.payload,
        }
        .payload,
    )
    .await
    .map(|doc| (StatusCode::CREATED, Json(doc)))
    .map_err(map_db_error)
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::Document>, AppError> {
    repository::get_document(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateDocument>,
) -> Result<Json<repository::Document>, AppError> {
    repository::update_document(&state.pool, &id, &input.payload)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_document(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

async fn list_document_audit(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<repository::AuditList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::list_document_audit(&state.pool, &id, query.limit, query.offset)
        .await
        .map(Json)
        .map_err(map_db_error)
}

fn map_db_error(error: anyhow::Error) -> AppError {
    if let Some(app) = error.downcast_ref::<AppError>() {
        return match app {
            AppError::BadRequest(msg) => AppError::BadRequest(msg.clone()),
            AppError::NotFound(msg) => AppError::NotFound(msg.clone()),
            AppError::Conflict(msg) => AppError::Conflict(msg.clone()),
            AppError::Internal(_) => AppError::Internal(anyhow::anyhow!("internal error")),
        };
    }
    let message = error.to_string();
    if message.contains("UNIQUE constraint failed") {
        return AppError::Conflict("duplicate key".into());
    }
    if message.contains("FOREIGN KEY constraint failed") {
        return AppError::BadRequest("unknown entity_id".into());
    }
    AppError::Internal(error)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "status": "ok" }))
}

async fn version() -> axum::Json<serde_json::Value> {
    axum::Json(json!({ "name": "logholizon-core", "version": env!("CARGO_PKG_VERSION") }))
}
