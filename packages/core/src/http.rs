use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
        .route(
            "/v1/meta/entities/{id}",
            get(get_entity).put(update_entity).delete(delete_entity),
        )
        .route("/v1/meta/entities/{id}/workflow", get(get_workflow))
        .route(
            "/v1/meta/entities/{id}/workflow/states",
            axum::routing::post(create_workflow_state),
        )
        .route(
            "/v1/meta/workflow/states/{id}",
            axum::routing::put(update_workflow_state).delete(delete_workflow_state),
        )
        .route(
            "/v1/meta/entities/{id}/workflow/transitions",
            axum::routing::post(create_workflow_transition),
        )
        .route(
            "/v1/meta/workflow/transitions/{id}",
            axum::routing::delete(delete_workflow_transition),
        )
        .route("/v1/meta/entities/{id}/export", get(export_documents))
        .route(
            "/v1/meta/entities/{id}/import/preview",
            axum::routing::post(preview_import),
        )
        .route(
            "/v1/meta/entities/{id}/import/confirm",
            axum::routing::post(confirm_import),
        )
        .route(
            "/v1/meta/entities/{id}/fields",
            axum::routing::post(create_field),
        )
        .route(
            "/v1/meta/fields/{id}",
            axum::routing::put(update_field).delete(delete_field),
        )
        .route(
            "/v1/meta/fields/{id}/options",
            axum::routing::post(create_field_option),
        )
        .route(
            "/v1/meta/options/{id}",
            axum::routing::put(update_field_option).delete(delete_field_option),
        )
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
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_dir: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct UpdateEntity {
    pub name: String,
    pub label: String,
}

async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateEntity>,
) -> Result<Json<repository::Entity>, AppError> {
    repository::update_entity(&state.pool, &id, &input.name, &input.label)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_entity(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateField {
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub is_status: bool,
}

async fn create_field(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateField>,
) -> Result<(StatusCode, Json<repository::Field>), AppError> {
    repository::create_field(
        &state.pool,
        &id,
        &input.name,
        &input.r#type,
        input.required,
        input.is_status,
    )
    .await
    .map(|field| (StatusCode::CREATED, Json(field)))
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateField {
    pub name: String,
    pub r#type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub is_status: bool,
}

async fn update_field(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateField>,
) -> Result<Json<repository::Field>, AppError> {
    repository::update_field(
        &state.pool,
        &id,
        &input.name,
        &input.r#type,
        input.required,
        input.is_status,
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn delete_field(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_field(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateFieldOption {
    pub value: String,
    pub label: String,
}

async fn create_field_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateFieldOption>,
) -> Result<(StatusCode, Json<repository::FieldOption>), AppError> {
    repository::create_field_option(&state.pool, &id, &input.value, &input.label)
        .await
        .map(|option| (StatusCode::CREATED, Json(option)))
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateFieldOption {
    pub value: String,
    pub label: String,
}

async fn update_field_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateFieldOption>,
) -> Result<Json<repository::FieldOption>, AppError> {
    repository::update_field_option(&state.pool, &id, &input.value, &input.label)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_field_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_field_option(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
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
pub struct CreateWorkflowState {
    pub name: String,
    pub label: String,
}

async fn create_workflow_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateWorkflowState>,
) -> Result<(StatusCode, Json<repository::WorkflowState>), AppError> {
    repository::create_workflow_state(&state.pool, &id, &input.name, &input.label)
        .await
        .map(|row| (StatusCode::CREATED, Json(row)))
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowState {
    pub label: String,
}

async fn update_workflow_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateWorkflowState>,
) -> Result<Json<repository::WorkflowState>, AppError> {
    repository::update_workflow_state(&state.pool, &id, &input.label)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_workflow_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_workflow_state(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowTransition {
    pub from_state: String,
    pub to_state: String,
    pub action: String,
}

async fn create_workflow_transition(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateWorkflowTransition>,
) -> Result<(StatusCode, Json<repository::WorkflowTransition>), AppError> {
    repository::create_workflow_transition(
        &state.pool,
        &id,
        &input.from_state,
        &input.to_state,
        &input.action,
    )
    .await
    .map(|row| (StatusCode::CREATED, Json(row)))
    .map_err(map_db_error)
}

async fn delete_workflow_transition(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_workflow_transition(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

async fn preview_import(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: String,
) -> Result<Json<repository::ImportPreview>, AppError> {
    repository::preview_documents_csv(&state.pool, &id, &body)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn confirm_import(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: String,
) -> Result<Json<repository::ImportResult>, AppError> {
    repository::confirm_documents_csv(&state.pool, &id, &body)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn export_documents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    let csv = repository::export_documents_csv(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    Ok(([("content-type", "text/csv; charset=utf-8")], csv).into_response())
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
    repository::list_documents(
        &state.pool,
        &query.entity_id,
        query.limit,
        query.offset,
        &repository::ListDocumentsFilter {
            search: query.search,
            status: query.status,
            sort_by: query.sort_by,
            sort_dir: query.sort_dir,
        },
    )
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
