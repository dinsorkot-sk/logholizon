use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{auth, dynamic_crud, error::AppError, http::AppState, repository};

fn role(user: &Option<Extension<auth::User>>) -> &str {
    user.as_ref().map(|u| u.0.role.as_str()).unwrap_or("admin")
}

fn actor(user: &Option<Extension<auth::User>>) -> Option<&str> {
    user.as_ref().map(|u| u.0.id.as_str())
}

#[derive(Debug, Deserialize, Default)]
pub struct DynamicListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_dir: Option<String>,
    #[serde(default)]
    pub view_id: Option<String>,
}

fn default_limit() -> i64 {
    50
}

pub async fn list(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity)): Path<(String, String)>,
    Query(q): Query<DynamicListQuery>,
) -> Result<Json<repository::DocumentList>, AppError> {
    if !(1..=100).contains(&q.limit) || q.offset < 0 {
        return Err(AppError::BadRequest(
            "limit must be 1..=100 and offset must be >= 0".into(),
        ));
    }
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let r = role(&user);
    repository::check_permission(&state.pool, &entity.id, r, false).await?;
    repository::list_documents_as_role(
        &state.pool,
        &entity.id,
        q.limit,
        q.offset,
        &repository::ListDocumentsFilter {
            search: q.search,
            status: None,
            sort_by: q.sort_by,
            sort_dir: q.sort_dir,
            view_id: q.view_id,
        },
        r,
    )
    .await
    .map(Json)
    .map_err(AppError::from)
}

#[derive(Debug, Deserialize)]
pub struct CreateBody {
    pub id: String,
    #[serde(default)]
    pub payload: Value,
}

pub async fn create(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity)): Path<(String, String)>,
    Json(body): Json<CreateBody>,
) -> Result<(StatusCode, Json<repository::Document>), AppError> {
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let r = role(&user);
    repository::check_permission(&state.pool, &entity.id, r, true).await?;
    repository::create_document_as_role(
        &state.pool,
        &body.id,
        &entity.id,
        &body.payload,
        actor(&user),
        r,
    )
    .await
    .map(|d| (StatusCode::CREATED, Json(d)))
    .map_err(AppError::from)
}

pub async fn get(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity, id)): Path<(String, String, String)>,
) -> Result<Json<repository::Document>, AppError> {
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let r = role(&user);
    repository::check_permission(&state.pool, &entity.id, r, false).await?;
    let doc = repository::get_document(&state.pool, &id).await?;
    if doc.entity_id != entity.id {
        return Err(AppError::NotFound(format!("document not found: {id}")));
    }
    let fields = repository::viewable_field_names(&state.pool, &entity.id, r).await?;
    Ok(Json(repository::Document {
        payload: repository::redact_payload(&doc.payload, &fields),
        ..doc
    }))
}
#[derive(Debug, Deserialize)]
pub struct UpdateBody {
    pub payload: Value,
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity, id)): Path<(String, String, String)>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<repository::Document>, AppError> {
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let r = role(&user);
    repository::check_permission(&state.pool, &entity.id, r, true).await?;
    let existing = repository::get_document(&state.pool, &id).await?;
    if existing.entity_id != entity.id {
        return Err(AppError::NotFound(format!("document not found: {id}")));
    }
    repository::update_document_as_role(
        &state.pool,
        &id,
        &body.payload,
        actor(&user),
        body.expected_updated_at.as_deref(),
        r,
    )
    .await
    .map(Json)
    .map_err(AppError::from)
}

pub async fn delete(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity, id)): Path<(String, String, String)>,
) -> Result<StatusCode, AppError> {
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let r = role(&user);
    repository::check_permission(&state.pool, &entity.id, r, true).await?;
    let existing = repository::get_document(&state.pool, &id).await?;
    if existing.entity_id != entity.id {
        return Err(AppError::NotFound(format!("document not found: {id}")));
    }
    repository::delete_document_as_role(&state.pool, &id, actor(&user), r).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteBody {
    pub ids: Vec<String>,
}

pub async fn bulk_delete(
    State(state): State<AppState>,
    user: Option<Extension<auth::User>>,
    Path((module, entity)): Path<(String, String)>,
    Json(body): Json<BulkDeleteBody>,
) -> Result<Json<Value>, AppError> {
    let entity = dynamic_crud::resolve_entity(&state.pool, &module, &entity).await?;
    let count = dynamic_crud::bulk_delete(
        &state.pool,
        &entity.id,
        &body.ids,
        actor(&user),
        role(&user),
    )
    .await?;
    Ok(Json(serde_json::json!({ "deleted": count })))
}
