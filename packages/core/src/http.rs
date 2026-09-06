use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::SqlitePool;

use crate::{
    auth, backup,
    error::AppError,
    repository::{self, CreateDocument, UpdateDocument},
    Config,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Config,
}

fn bearer_token(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))
}

async fn require_user(state: &AppState, headers: &HeaderMap) -> Result<auth::User, AppError> {
    let token = bearer_token(headers)?;
    auth::user_for_token(&state.pool, &token)
        .await
        .map_err(AppError::from)
}

/// Enforce authentication on every `/v1/*` route except the public whitelist.
/// `/v1/meta/*` and `/v1/admin/*` additionally require the admin role.
async fn auth_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let is_public = path == "/health"
        || path == "/v1/version"
        || path == "/v1/auth/register"
        || path == "/v1/auth/login"
        || path == "/v1/auth/status";
    if is_public {
        return Ok(next.run(request).await);
    }

    let user = require_user(&state, request.headers()).await?;
    if (path.starts_with("/v1/meta/") || path.starts_with("/v1/admin/")) && user.role != "admin" {
        return Err(AppError::Forbidden("admin role required".into()));
    }

    let mut request = request;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub fn router(config: &Config, pool: SqlitePool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/version", get(version))
        .route("/v1/auth/register", axum::routing::post(auth_register))
        .route("/v1/auth/login", axum::routing::post(auth_login))
        .route("/v1/auth/logout", axum::routing::post(auth_logout))
        .route("/v1/auth/me", get(auth_me))
        .route("/v1/auth/status", get(auth_status))
        .route("/v1/admin/users", get(list_users).post(create_user))
        .route(
            "/v1/admin/users/{id}",
            axum::routing::put(update_user).delete(delete_user),
        )
        .route(
            "/v1/admin/users/{id}/reset-password",
            axum::routing::post(reset_user_password),
        )
        .route("/v1/admin/status", get(admin_status))
        .route("/v1/admin/backup", axum::routing::post(admin_backup))
        .route("/v1/admin/backups", get(admin_list_backups))
        .route("/v1/admin/backups/{name}", get(admin_download_backup))
        .route("/v1/admin/restore", axum::routing::post(admin_restore))
        .route("/v1/admin/restart", axum::routing::post(admin_restart))
        .route("/v1/meta/entities", get(list_entities).post(create_entity))
        .route(
            "/v1/meta/entities/{id}",
            get(get_entity).put(update_entity).delete(delete_entity),
        )
        .route("/v1/meta/entities/{id}/workflow", get(get_workflow))
        .route(
            "/v1/meta/entities/{id}/permissions",
            get(get_entity_permissions).put(update_entity_permissions),
        )
        .route(
            "/v1/meta/entities/{id}/field-permissions",
            get(get_field_permissions).put(update_field_permissions),
        )
        .route(
            "/v1/meta/entities/{id}/views",
            get(list_entity_views).post(create_entity_view),
        )
        .route(
            "/v1/meta/views/{id}",
            get(get_entity_view).delete(delete_entity_view),
        )
        .route(
            "/v1/meta/entities/{id}/form-layout",
            get(get_entity_form_layout).put(update_entity_form_layout),
        )
        .route(
            "/v1/meta/entities/{id}/notification-rules",
            get(list_notification_rules).post(create_notification_rule),
        )
        .route(
            "/v1/meta/notification-rules/{id}",
            axum::routing::put(update_notification_rule).delete(delete_notification_rule),
        )
        .route(
            "/v1/admin/notification-deliveries",
            get(list_notification_deliveries),
        )
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
        .route("/v1/entities", get(list_entities_for_user))
        .route("/v1/entities/export", get(export_workbook_for_user))
        .route(
            "/v1/entities/import/preview",
            axum::routing::post(preview_workbook_import_for_user),
        )
        .route(
            "/v1/entities/import/confirm",
            axum::routing::post(confirm_workbook_import_for_user),
        )
        .route("/v1/entities/{id}", get(get_entity_for_user))
        .route("/v1/entities/{id}/workflow", get(get_workflow_for_user))
        .route("/v1/entities/{id}/views", get(list_entity_views_for_user))
        .route("/v1/views/{id}", get(get_entity_view_for_user))
        .route(
            "/v1/entities/{id}/form-layout",
            get(get_entity_form_layout_for_user),
        )
        .route("/v1/entities/{id}/export", get(export_documents_for_user))
        .route(
            "/v1/entities/{id}/import/preview",
            axum::routing::post(preview_import_for_user),
        )
        .route(
            "/v1/entities/{id}/import/confirm",
            axum::routing::post(confirm_import_for_user),
        )
        .route("/v1/documents", get(list_documents).post(create_document))
        .route(
            "/v1/documents/{id}",
            get(get_document)
                .put(update_document)
                .delete(delete_document),
        )
        .route("/v1/documents/{id}/audit", get(list_document_audit))
        .route("/v1/audit", get(list_global_audit))
        .route(
            "/v1/documents/{id}/transition",
            axum::routing::post(transition_document),
        )
        .route("/v1/dashboard/counts", get(dashboard_counts))
        .route("/v1/dashboard/pm", get(dashboard_pm))
        .layer(middleware::from_fn_with_state(
            AppState {
                pool: pool.clone(),
                config: config.clone(),
            },
            auth_middleware,
        ))
        .with_state(AppState {
            pool,
            config: config.clone(),
        })
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
    #[serde(default)]
    pub view_id: Option<String>,
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

async fn get_entity_permissions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::EntityPermission>>, AppError> {
    repository::get_entity_permissions(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionsRequest {
    pub permissions: Vec<PermissionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionEntry {
    pub role: String,
    #[serde(default = "default_true")]
    pub can_view: bool,
    #[serde(default)]
    pub can_edit: bool,
}

fn default_true() -> bool {
    true
}

async fn update_entity_permissions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePermissionsRequest>,
) -> Result<Json<Vec<repository::EntityPermission>>, AppError> {
    let entries: Vec<(String, bool, bool)> = input
        .permissions
        .into_iter()
        .map(|p| (p.role, p.can_view, p.can_edit))
        .collect();
    repository::update_entity_permissions(&state.pool, &id, &entries)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_field_permissions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::FieldPermission>>, AppError> {
    repository::get_field_permissions(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateFieldPermissionsRequest {
    pub permissions: Vec<FieldPermissionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FieldPermissionEntry {
    pub field_id: String,
    pub role: String,
    #[serde(default = "default_true")]
    pub can_view: bool,
    #[serde(default)]
    pub can_edit: bool,
}

async fn update_field_permissions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateFieldPermissionsRequest>,
) -> Result<Json<Vec<repository::FieldPermission>>, AppError> {
    let entries: Vec<(String, String, bool, bool)> = input
        .permissions
        .into_iter()
        .map(|p| (p.field_id, p.role, p.can_view, p.can_edit))
        .collect();
    repository::update_field_permissions(&state.pool, &id, &entries)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn list_entity_views(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::EntityView>>, AppError> {
    repository::list_entity_views(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateViewRequest {
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

async fn create_entity_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateViewRequest>,
) -> Result<(StatusCode, Json<repository::EntityView>), AppError> {
    repository::create_entity_view(&state.pool, &id, &input.name, &input.config)
        .await
        .map(|view| (StatusCode::CREATED, Json(view)))
        .map_err(map_db_error)
}

async fn get_entity_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::EntityView>, AppError> {
    repository::get_entity_view(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_entity_view(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_entity_view(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

async fn list_notification_rules(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::NotificationRule>>, AppError> {
    repository::list_notification_rules(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRule {
    #[serde(default = "default_rule_trigger")]
    pub trigger: String,
    pub target_url: String,
    #[serde(default = "default_rule_active")]
    pub active: bool,
}

fn default_rule_trigger() -> String {
    "transition".to_string()
}

fn default_rule_active() -> bool {
    true
}

async fn create_notification_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateNotificationRule>,
) -> Result<(StatusCode, Json<repository::NotificationRule>), AppError> {
    repository::create_notification_rule(
        &state.pool,
        &id,
        &input.trigger,
        &input.target_url,
        input.active,
    )
    .await
    .map(|rule| (StatusCode::CREATED, Json(rule)))
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRule {
    pub trigger: Option<String>,
    pub target_url: Option<String>,
    pub active: Option<bool>,
}

async fn update_notification_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateNotificationRule>,
) -> Result<Json<repository::NotificationRule>, AppError> {
    repository::update_notification_rule(
        &state.pool,
        &id,
        input.trigger.as_deref(),
        input.target_url.as_deref(),
        input.active,
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn delete_notification_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_notification_rule(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

async fn list_notification_deliveries(
    State(state): State<AppState>,
    Query(query): Query<ListDeliveriesQuery>,
) -> Result<Json<repository::NotificationDeliveryList>, AppError> {
    if !(1..=100).contains(&query.limit) || query.offset < 0 {
        return Err(AppError::BadRequest("invalid pagination".into()));
    }
    repository::list_notification_deliveries(&state.pool, query.limit, query.offset)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_entity_form_layout(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::FormLayout>, AppError> {
    repository::get_entity_form_layout(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateFormLayoutRequest {
    #[serde(default)]
    pub config: serde_json::Value,
}

async fn update_entity_form_layout(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateFormLayoutRequest>,
) -> Result<Json<repository::FormLayout>, AppError> {
    repository::update_entity_form_layout(&state.pool, &id, &input.config)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_entity_form_layout_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::FormLayout>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::get_entity_form_layout(&state.pool, &id)
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
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    body: String,
) -> Result<Json<repository::ImportResult>, AppError> {
    repository::confirm_documents_csv(&state.pool, &id, &body, current_actor(&user).as_deref())
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
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

async fn transition_document(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<TransitionRequest>,
) -> Result<Json<repository::Document>, AppError> {
    if input.action.trim().is_empty() {
        return Err(AppError::BadRequest("action is required".into()));
    }
    let existing = repository::get_document(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &existing.entity_id, &current_role(&user), true)
        .await
        .map_err(map_db_error)?;
    repository::transition_document_as_role(
        &state.pool,
        &id,
        &input.action,
        current_actor(&user).as_deref(),
        input.expected_updated_at.as_deref(),
        &current_role(&user),
    )
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
    user: Option<axum::extract::Extension<auth::User>>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<Vec<repository::StatusCount>>, AppError> {
    if query.entity_id.trim().is_empty() {
        return Err(AppError::BadRequest("entity_id is required".into()));
    }
    repository::check_permission(&state.pool, &query.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::count_documents_by_status_as_role(
        &state.pool,
        &query.entity_id,
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn dashboard_pm(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<repository::PmSummary>, AppError> {
    if query.entity_id.trim().is_empty() {
        return Err(AppError::BadRequest("entity_id is required".into()));
    }
    repository::check_permission(&state.pool, &query.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::pm_summary_as_role(&state.pool, &query.entity_id, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

fn current_role(user: &Option<axum::extract::Extension<auth::User>>) -> String {
    user.as_ref()
        .map(|u| u.role.clone())
        .unwrap_or_else(|| "user".to_string())
}

fn current_actor(user: &Option<axum::extract::Extension<auth::User>>) -> Option<String> {
    user.as_ref().map(|u| u.username.clone())
}

async fn list_entities_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
) -> Result<Json<Vec<repository::Entity>>, AppError> {
    repository::list_entities_for_role(&state.pool, &current_role(&user))
        .await
        .map(Json)
        .map_err(AppError::from)
}

/// Non-admin entity detail for the dynamic record UI.
/// Returns the entity plus the caller's own permission row.
async fn get_entity_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::EntityWithPermission>, AppError> {
    repository::get_entity_with_permission(&state.pool, &id, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_workflow_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::WorkflowDefinition>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::get_workflow(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn list_entity_views_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::EntityView>>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::list_entity_views(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_entity_view_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::EntityView>, AppError> {
    let view = repository::get_entity_view(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &view.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    Ok(Json(view))
}

async fn export_documents_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    let csv = repository::export_documents_csv_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map_err(map_db_error)?;
    Ok(([("content-type", "text/csv; charset=utf-8")], csv).into_response())
}

async fn preview_import_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    body: String,
) -> Result<Json<repository::ImportPreview>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::preview_documents_csv_as_role(&state.pool, &id, &body, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn confirm_import_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    body: String,
) -> Result<Json<repository::ImportResult>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), true)
        .await
        .map_err(map_db_error)?;
    repository::confirm_documents_csv_as_role(
        &state.pool,
        &id,
        &body,
        current_actor(&user).as_deref(),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

const XLSX_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// Export every entity visible to the caller as one `.xlsx` workbook.
async fn export_workbook_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
) -> Result<axum::response::Response, AppError> {
    let bytes = repository::export_workbook_xlsx(&state.pool, &current_role(&user))
        .await
        .map_err(map_db_error)?;
    Ok(([("content-type", XLSX_CONTENT_TYPE)], bytes).into_response())
}

/// Preview a whole-workbook `.xlsx` import: one entry per sheet.
async fn preview_workbook_import_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    body: axum::body::Bytes,
) -> Result<Json<repository::MultiImportPreview>, AppError> {
    repository::preview_workbook_xlsx(&state.pool, &body, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

/// Confirm a whole-workbook `.xlsx` import atomically.
async fn confirm_workbook_import_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    body: axum::body::Bytes,
) -> Result<Json<repository::MultiImportResult>, AppError> {
    repository::confirm_workbook_xlsx(
        &state.pool,
        &body,
        current_actor(&user).as_deref(),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn list_documents(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<repository::DocumentList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::check_permission(&state.pool, &query.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::list_documents_as_role(
        &state.pool,
        &query.entity_id,
        query.limit,
        query.offset,
        &repository::ListDocumentsFilter {
            search: query.search,
            status: query.status,
            sort_by: query.sort_by,
            sort_dir: query.sort_dir,
            view_id: query.view_id,
        },
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn create_document(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Json(input): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<repository::Document>), AppError> {
    repository::check_permission(&state.pool, &input.entity_id, &current_role(&user), true)
        .await
        .map_err(map_db_error)?;
    repository::create_document_as_role(
        &state.pool,
        &input.id,
        &input.entity_id,
        &CreateDocument {
            payload: input.payload,
        }
        .payload,
        current_actor(&user).as_deref(),
        &current_role(&user),
    )
    .await
    .map(|doc| (StatusCode::CREATED, Json(doc)))
    .map_err(map_db_error)
}

async fn get_document(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Document>, AppError> {
    let doc = repository::get_document(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &doc.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    let viewable =
        repository::viewable_field_names(&state.pool, &doc.entity_id, &current_role(&user))
            .await
            .map_err(map_db_error)?;
    let doc = repository::Document {
        payload: repository::redact_payload(&doc.payload, &viewable),
        ..doc
    };
    Ok(Json(doc))
}

async fn update_document(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateDocument>,
) -> Result<Json<repository::Document>, AppError> {
    let existing = repository::get_document(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &existing.entity_id, &current_role(&user), true)
        .await
        .map_err(map_db_error)?;
    repository::update_document_as_role(
        &state.pool,
        &id,
        &input.payload,
        current_actor(&user).as_deref(),
        input.expected_updated_at.as_deref(),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn delete_document(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let existing = repository::get_document(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &existing.entity_id, &current_role(&user), true)
        .await
        .map_err(map_db_error)?;
    repository::delete_document(&state.pool, &id, current_actor(&user).as_deref())
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
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<repository::AuditList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::list_document_audit_as_role(
        &state.pool,
        &id,
        query.limit,
        query.offset,
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct GlobalAuditQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
}

async fn list_global_audit(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Query(query): Query<GlobalAuditQuery>,
) -> Result<Json<repository::GlobalAuditList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::list_global_audit_as_role(
        &state.pool,
        query.limit,
        query.offset,
        &repository::GlobalAuditFilter {
            entity_id: query.entity_id,
            action: query.action,
            search: query.search,
        },
        &current_role(&user),
    )
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
            AppError::Unauthorized(msg) => AppError::Unauthorized(msg.clone()),
            AppError::Forbidden(msg) => AppError::Forbidden(msg.clone()),
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

// --- Auth ---

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

async fn auth_register(
    State(state): State<AppState>,
    Json(input): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<auth::User>), AppError> {
    auth::register(&state.pool, &input.username, &input.password)
        .await
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

async fn auth_login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<auth::Session>, AppError> {
    auth::login(&state.pool, &input.username, &input.password)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = bearer_token(&headers)?;
    auth::logout(&state.pool, &token)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "message": "logged out" })))
}

async fn auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<auth::User>, AppError> {
    require_user(&state, &headers).await.map(Json)
}

async fn auth_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let has_users = auth::has_users(&state.pool).await.map_err(AppError::from)?;
    Ok(Json(json!({ "has_users": has_users })))
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<auth::UserRow>>, AppError> {
    auth::list_users(&state.pool)
        .await
        .map(Json)
        .map_err(AppError::from)
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    #[serde(default = "default_user_role")]
    pub role: String,
}

fn default_user_role() -> String {
    "user".to_string()
}

async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<auth::User>), AppError> {
    auth::create_user(&state.pool, &input.username, &input.password, &input.role)
        .await
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub role: String,
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUserRequest>,
) -> Result<Json<auth::User>, AppError> {
    auth::update_user_role(&state.pool, &id, &input.role)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    auth::delete_user(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub password: String,
}

async fn reset_user_password(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ResetPasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth::reset_password(&state.pool, &id, &input.password)
        .await
        .map(|()| Json(json!({ "message": "password reset" })))
        .map_err(map_db_error)
}

// --- Admin: status / backup / restore ---

fn database_path(state: &AppState) -> Result<std::path::PathBuf, AppError> {
    crate::db::database_path(&state.config.database_url)
        .map(std::path::Path::to_path_buf)
        .map_err(AppError::Internal)
}

fn backups_dir(state: &AppState) -> Result<std::path::PathBuf, AppError> {
    let db_path = database_path(state)?;
    Ok(db_path
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| std::path::PathBuf::from("backups")))
}

async fn admin_status(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let integrity = crate::db::integrity_check(&state.pool)
        .await
        .map_err(AppError::from)?;
    let entities: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _meta_entity")
        .fetch_one(&state.pool)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    let documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _doc")
        .fetch_one(&state.pool)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "database_path": database_path(&state)?.to_string_lossy(),
        "integrity": integrity,
        "entities": entities,
        "documents": documents,
        "backup_interval_hours": state.config.backup_interval_hours,
        "backup_keep": state.config.backup_keep,
    })))
}

async fn admin_backup(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let dir = backups_dir(&state)?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let destination = dir.join(format!("core-{timestamp}.db"));
    backup::backup(&state.pool, &destination)
        .await
        .map_err(AppError::from)?;
    Ok(Json(json!({ "path": destination.to_string_lossy() })))
}

async fn admin_list_backups(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let dir = backups_dir(&state)?;
    let mut items = Vec::new();
    if dir.is_dir() {
        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(|error| AppError::Internal(error.into()))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|error| AppError::Internal(error.into()))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("db") {
                continue;
            }
            let metadata = entry
                .metadata()
                .await
                .map_err(|error| AppError::Internal(error.into()))?;
            items.push(json!({
                "name": path.file_name().and_then(|n| n.to_str()).unwrap_or_default(),
                "size": metadata.len(),
                "modified": metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            }));
        }
    }
    items.sort_by(|a, b| b["name"].as_str().cmp(&a["name"].as_str()));
    Ok(Json(json!({ "items": items })))
}

async fn admin_download_backup(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<axum::response::Response, AppError> {
    if name.contains(['/', '\\', '.', ':']) || !name.ends_with(".db") {
        return Err(AppError::BadRequest("invalid backup name".into()));
    }
    let path = backups_dir(&state)?.join(&name);
    if !path.is_file() {
        return Err(AppError::NotFound(format!("backup not found: {name}")));
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(([("content-type", "application/octet-stream")], bytes).into_response())
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub path: String,
    #[serde(default)]
    pub force: bool,
}

async fn admin_restore(
    State(state): State<AppState>,
    Json(input): Json<RestoreRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !input.force {
        return Err(AppError::BadRequest("restore requires force=true".into()));
    }
    let source = std::path::PathBuf::from(&input.path);
    backup::validate(&source).await.map_err(AppError::from)?;
    let db_path = database_path(&state)?;
    let staging = db_path
        .parent()
        .map(|p| p.join("restore-pending.db"))
        .unwrap_or_else(|| std::path::PathBuf::from("restore-pending.db"));
    tokio::fs::copy(&source, &staging)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(Json(json!({
        "message": "Restore staged. Restart core to apply.",
        "staged": staging.to_string_lossy(),
    })))
}

async fn admin_restart(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        std::process::exit(0);
    });
    Ok(Json(json!({ "message": "Core is restarting." })))
}
