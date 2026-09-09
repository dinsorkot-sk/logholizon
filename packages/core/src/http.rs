use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::{
    auth, backup,
    error::AppError,
    module_lifecycle, relation,
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
        .route("/v1/admin/roles", get(list_roles).post(create_role))
        .route(
            "/v1/admin/roles/{id}",
            axum::routing::put(update_role).delete(delete_role),
        )
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
        .route("/v1/modules", get(list_modules).post(create_module))
        .route(
            "/v1/modules/{id}",
            get(get_module).put(update_module).delete(delete_module),
        )
        .route("/v1/modules/{id}/manifest", get(get_module_manifest))
        .route(
            "/v1/modules/{id}/publish",
            axum::routing::post(publish_module),
        )
        .route(
            "/v1/modules/{id}/review",
            axum::routing::post(review_module),
        )
        .route(
            "/v1/modules/{id}/enable",
            axum::routing::post(enable_module),
        )
        .route(
            "/v1/modules/{id}/disable",
            axum::routing::post(disable_module),
        )
        .route(
            "/v1/modules/{id}/archive",
            axum::routing::post(archive_module),
        )
        .route(
            "/v1/modules/{id}/restore",
            axum::routing::post(restore_module),
        )
        .route("/v1/modules/{id}/versions", get(list_module_versions))
        .route(
            "/v1/modules/{id}/rollback",
            axum::routing::post(rollback_module),
        )
        .route(
            "/v1/meta/entities/{id}/automations",
            get(list_automations).post(create_automation),
        )
        .route(
            "/v1/meta/automations/{id}",
            axum::routing::put(update_automation).delete(delete_automation),
        )
        .route(
            "/v1/meta/automations/{id}/executions",
            get(list_automation_executions),
        )
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
            "/v1/meta/entities/{id}/actions",
            get(list_module_actions).post(create_module_action),
        )
        .route(
            "/v1/meta/actions/{id}",
            axum::routing::delete(delete_module_action),
        )
        .route("/v1/meta/entities/{id}/events", get(list_events))
        .route(
            "/v1/meta/notification-rules/{id}",
            axum::routing::put(update_notification_rule).delete(delete_notification_rule),
        )
        .route(
            "/v1/admin/notification-deliveries",
            get(list_notification_deliveries),
        )
        .route(
            "/v1/meta/entities/{id}/reports",
            get(list_reports).post(create_report),
        )
        .route(
            "/v1/meta/reports/{id}",
            get(get_report).delete(delete_report),
        )
        .route("/v1/entities/{id}/reports", get(list_reports_for_user))
        .route("/v1/reports/{id}", get(get_report_for_user))
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
            "/v1/meta/entities/{id}/relations",
            get(list_relations).post(create_relation),
        )
        .route(
            "/v1/meta/relations/{id}",
            get(get_relation)
                .put(update_relation)
                .delete(delete_relation),
        )
        .route(
            "/v1/meta/relations/{id}/links/{source_doc_id}",
            get(list_relation_links).put(set_relation_links),
        )
        .route(
            "/v1/entities/{id}/relations/{relation_id}",
            get(related_documents),
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
        .route("/v1/entities/{id}/options", get(entity_options))
        .route("/v1/entities/{id}/workflow", get(get_workflow_for_user))
        .route(
            "/v1/entities/{id}/views",
            get(list_entity_views_for_user).post(create_entity_view_for_user),
        )
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
        .route(
            "/v1/modules/{module}/entities/{entity}",
            get(crate::dynamic_http::list).post(crate::dynamic_http::create),
        )
        .route(
            "/v1/modules/{module}/entities/{entity}/{id}",
            get(crate::dynamic_http::get)
                .put(crate::dynamic_http::update)
                .delete(crate::dynamic_http::delete),
        )
        .route(
            "/v1/modules/{module}/entities/{entity}/bulk-delete",
            axum::routing::post(crate::dynamic_http::bulk_delete),
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
            "/v1/documents/{id}/comments",
            get(list_doc_comments).post(create_doc_comment),
        )
        .route(
            "/v1/documents/{id}/followers",
            get(list_doc_followers).post(toggle_doc_follower),
        )
        .route(
            "/v1/documents/{id}/activities",
            get(list_doc_activities).post(create_doc_activity),
        )
        .route(
            "/v1/activities/{id}/toggle",
            axum::routing::post(toggle_doc_activity),
        )
        .route(
            "/v1/documents/{id}/attachments",
            get(list_doc_attachments).post(upload_doc_attachment),
        )
        .route(
            "/v1/attachments/{id}",
            get(download_doc_attachment).delete(delete_doc_attachment),
        )
        .route("/v1/audit", get(list_global_audit))
        .route(
            "/v1/documents/{id}/transition",
            axum::routing::post(transition_document),
        )
        .route(
            "/v1/documents/{id}/workflow-history",
            get(get_workflow_history),
        )
        .route("/v1/dashboard/counts", get(dashboard_counts))
        .route("/v1/dashboard/pm", get(dashboard_pm))
        .route("/v1/reports/aggregate", get(report_aggregate))
        .route(
            "/v1/entities/{id}/actions/{action_id}",
            axum::routing::post(execute_module_action),
        )
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

#[derive(Debug, Deserialize)]
pub struct EntityOptionsQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default = "default_options_limit")]
    pub limit: i64,
}

fn default_options_limit() -> i64 {
    50
}

async fn entity_options(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Query(query): Query<EntityOptionsQuery>,
) -> Result<Json<Vec<repository::EntityOption>>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest(
            "limit must be between 1 and 100".into(),
        ));
    }
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::list_entity_options(
        &state.pool,
        &id,
        &current_role(&user),
        query.search.as_deref(),
        query.limit,
    )
    .await
    .map(Json)
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
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: Option<Value>,
}

async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateEntity>,
) -> Result<Json<repository::Entity>, AppError> {
    repository::update_entity(
        &state.pool,
        &id,
        &input.name,
        &input.label,
        input.module.as_deref(),
    )
    .await
    .map_err(map_db_error)?;
    repository::update_entity_metadata(
        &state.pool,
        &id,
        input.description.as_deref(),
        input.settings.as_ref(),
    )
    .await
    .map(|detail| {
        Json(repository::Entity {
            id: detail.id,
            name: detail.name,
            label: detail.label,
            description: detail.description,
            settings: detail.settings,
            module: detail.module,
        })
    })
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
    #[serde(default)]
    pub ref_entity: Option<String>,
    #[serde(default)]
    pub computed_expr: Option<String>,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub min_length: Option<i64>,
    #[serde(default)]
    pub max_length: Option<i64>,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub auto_number_prefix: Option<String>,
    #[serde(default)]
    pub auto_number_width: Option<i64>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub searchable: bool,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub indexed: bool,
    #[serde(default)]
    pub precision: Option<i64>,
    #[serde(default)]
    pub help_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRelation {
    source_field_id: Option<String>,
    target_entity_id: String,
    target_field_id: Option<String>,
    name: String,
    relation_type: String,
    #[serde(default = "default_relation_delete")]
    on_delete: String,
}
fn default_relation_delete() -> String {
    "restrict".into()
}
#[derive(Debug, Deserialize)]
struct UpdateRelation {
    source_field_id: Option<String>,
    target_field_id: Option<String>,
    name: String,
    relation_type: String,
    on_delete: String,
}
#[derive(Debug, Deserialize)]
struct RelationLinksInput {
    target_doc_ids: Vec<String>,
}
async fn list_relations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<relation::Relation>>, AppError> {
    relation::list_relations(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}
async fn create_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateRelation>,
) -> Result<(StatusCode, Json<relation::Relation>), AppError> {
    relation::create_relation(
        &state.pool,
        &id,
        input.source_field_id.as_deref(),
        &input.target_entity_id,
        input.target_field_id.as_deref(),
        &input.name,
        &input.relation_type,
        &input.on_delete,
    )
    .await
    .map(|r| (StatusCode::CREATED, Json(r)))
    .map_err(map_db_error)
}
async fn get_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<relation::Relation>, AppError> {
    relation::get_relation(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}
async fn update_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateRelation>,
) -> Result<Json<relation::Relation>, AppError> {
    relation::update_relation(
        &state.pool,
        &id,
        &input.name,
        &input.relation_type,
        &input.on_delete,
        input.source_field_id.as_deref(),
        input.target_field_id.as_deref(),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}
async fn delete_relation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    relation::delete_relation(&state.pool, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}
async fn list_relation_links(
    State(state): State<AppState>,
    Path((relation_id, source_doc_id)): Path<(String, String)>,
) -> Result<Json<Vec<relation::RelationLink>>, AppError> {
    relation::list_links(&state.pool, &relation_id, &source_doc_id)
        .await
        .map(Json)
        .map_err(map_db_error)
}
async fn set_relation_links(
    State(state): State<AppState>,
    Path((relation_id, source_doc_id)): Path<(String, String)>,
    Json(input): Json<RelationLinksInput>,
) -> Result<Json<Vec<relation::RelationLink>>, AppError> {
    relation::set_links(
        &state.pool,
        &relation_id,
        &source_doc_id,
        &input.target_doc_ids,
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}
async fn related_documents(
    State(state): State<AppState>,
    Path((id, relation_id)): Path<(String, String)>,
) -> Result<Json<Vec<Value>>, AppError> {
    relation::related_documents(&state.pool, &relation_id, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}
async fn create_field(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateField>,
) -> Result<(StatusCode, Json<repository::Field>), AppError> {
    repository::create_field_with_rules(
        &state.pool,
        &id,
        &input.name,
        &input.r#type,
        input.required,
        input.is_status,
        input.ref_entity.as_deref(),
        input.computed_expr.as_deref(),
        &repository::FieldRules {
            is_unique: input.is_unique,
            min_value: input.min_value,
            max_value: input.max_value,
            pattern: input.pattern.clone(),
            min_length: input.min_length,
            max_length: input.max_length,
            default_value: input.default_value.clone(),
            auto_number_prefix: input.auto_number_prefix.clone(),
            auto_number_width: input.auto_number_width,
            label: input.label.clone(),
            description: input.description.clone(),
            readonly: input.readonly,
            hidden: input.hidden,
            searchable: input.searchable,
            sortable: input.sortable,
            filterable: input.filterable,
            indexed: input.indexed,
            precision: input.precision,
            help_text: input.help_text.clone(),
        },
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
    #[serde(default)]
    pub ref_entity: Option<String>,
    #[serde(default)]
    pub computed_expr: Option<String>,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub min_length: Option<i64>,
    #[serde(default)]
    pub max_length: Option<i64>,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub auto_number_prefix: Option<String>,
    #[serde(default)]
    pub auto_number_width: Option<i64>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub searchable: bool,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub indexed: bool,
    #[serde(default)]
    pub precision: Option<i64>,
    #[serde(default)]
    pub help_text: Option<String>,
}

async fn update_field(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateField>,
) -> Result<Json<repository::Field>, AppError> {
    repository::update_field_with_rules(
        &state.pool,
        &id,
        &input.name,
        &input.r#type,
        input.required,
        input.is_status,
        input.ref_entity.as_deref(),
        input.computed_expr.as_deref(),
        &repository::FieldRules {
            is_unique: input.is_unique,
            min_value: input.min_value,
            max_value: input.max_value,
            pattern: input.pattern.clone(),
            min_length: input.min_length,
            max_length: input.max_length,
            default_value: input.default_value.clone(),
            auto_number_prefix: input.auto_number_prefix.clone(),
            auto_number_width: input.auto_number_width,
            label: input.label.clone(),
            description: input.description.clone(),
            readonly: input.readonly,
            hidden: input.hidden,
            searchable: input.searchable,
            sortable: input.sortable,
            filterable: input.filterable,
            indexed: input.indexed,
            precision: input.precision,
            help_text: input.help_text.clone(),
        },
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

// --- User-defined modules: registry, manifest, publish lifecycle ---

#[derive(Debug, Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub definition: serde_json::Value,
}

async fn list_modules(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
) -> Result<Json<Vec<repository::Module>>, AppError> {
    let user = user.as_ref().map(|u| u.0.clone());
    let (owner, role) = match &user {
        Some(user) => (user.username.as_str(), user.role.as_str()),
        None => ("", "user"),
    };
    repository::list_modules(&state.pool, owner, role)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn create_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Json(input): Json<CreateModuleRequest>,
) -> Result<(StatusCode, Json<repository::Module>), AppError> {
    let definition = if input.definition.is_null() {
        serde_json::json!({"entities": []})
    } else {
        input.definition
    };
    repository::create_module(
        &state.pool,
        &input.name,
        &input.label,
        input.description.as_deref(),
        input.icon.as_deref(),
        input.color.as_deref(),
        &current_owner(&user),
        &definition,
        current_actor(&user).as_deref(),
    )
    .await
    .map(|module| (StatusCode::CREATED, Json(module)))
    .map_err(map_db_error)
}

async fn get_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    repository::get_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateModuleRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub definition: Option<serde_json::Value>,
}

async fn update_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<UpdateModuleRequest>,
) -> Result<Json<repository::Module>, AppError> {
    repository::update_module_draft(
        &state.pool,
        &id,
        input.label.as_deref(),
        input.description.as_deref(),
        input.icon.as_deref(),
        input.color.as_deref(),
        input.definition.as_ref(),
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn delete_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(|()| StatusCode::NO_CONTENT)
    .map_err(map_db_error)
}

async fn get_module_manifest(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    repository::get_module_manifest(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn review_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    module_lifecycle::submit_module_for_review(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn enable_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    module_lifecycle::enable_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn disable_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    module_lifecycle::disable_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn publish_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    repository::publish_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn archive_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    repository::archive_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn restore_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Module>, AppError> {
    repository::restore_module(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn list_module_versions(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::ModuleVersion>>, AppError> {
    repository::list_module_versions(
        &state.pool,
        &id,
        &current_owner(&user),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct RollbackModuleRequest {
    pub version: i64,
}

async fn rollback_module(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<RollbackModuleRequest>,
) -> Result<Json<repository::Module>, AppError> {
    repository::rollback_module(
        &state.pool,
        &id,
        input.version,
        &current_owner(&user),
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn list_automations(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::Automation>>, AppError> {
    repository::list_automations(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateAutomationRequest {
    pub trigger: String,
    #[serde(default = "default_automation_action")]
    pub action: String,
    pub target_url: String,
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_automation_action() -> String {
    "webhook".to_string()
}

async fn create_automation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateAutomationRequest>,
) -> Result<(StatusCode, Json<repository::Automation>), AppError> {
    repository::create_automation(
        &state.pool,
        &id,
        &input.trigger,
        &input.action,
        &input.target_url,
        input.active,
    )
    .await
    .map(|automation| (StatusCode::CREATED, Json(automation)))
    .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct UpdateAutomationRequest {
    pub condition: Option<String>,
    pub schedule: Option<String>,
    pub actions: Option<Value>,
    pub max_attempts: Option<i64>,
    pub active: Option<bool>,
}

async fn update_automation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAutomationRequest>,
) -> Result<Json<repository::Automation>, AppError> {
    crate::automation::update(
        &state.pool,
        &id,
        input.condition.as_deref(),
        input.schedule.as_deref(),
        input.actions.as_ref(),
        input.max_attempts,
        input.active,
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn list_automation_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::automation::AutomationExecution>>, AppError> {
    let rows=sqlx::query("SELECT id,automation_id,event_id,document_id,status,attempt,error,result,scheduled_at,started_at,finished_at,created_at FROM _automation_execution WHERE automation_id=? ORDER BY created_at DESC LIMIT 100").bind(id).fetch_all(&state.pool).await.map_err(|e| map_db_error(e.into()))?;
    use sqlx::Row;
    let out = rows
        .into_iter()
        .map(|r| crate::automation::AutomationExecution {
            id: r.try_get("id").unwrap(),
            automation_id: r.try_get("automation_id").unwrap(),
            event_id: r.try_get("event_id").unwrap(),
            document_id: r.try_get("document_id").unwrap(),
            status: r.try_get("status").unwrap(),
            attempt: r.try_get("attempt").unwrap(),
            error: r.try_get("error").unwrap(),
            result: serde_json::from_str(r.try_get::<String, _>("result").unwrap().as_str())
                .unwrap_or(Value::Object(Default::default())),
            scheduled_at: r.try_get("scheduled_at").unwrap(),
            started_at: r.try_get("started_at").unwrap(),
            finished_at: r.try_get("finished_at").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
        })
        .collect();
    Ok(Json(out))
}
async fn delete_automation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_automation(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

fn current_owner(user: &Option<axum::extract::Extension<auth::User>>) -> String {
    user.as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_default()
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

async fn list_reports(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::Report>>, AppError> {
    repository::list_reports(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateReportRequest {
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

async fn create_report(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<CreateReportRequest>,
) -> Result<(StatusCode, Json<repository::Report>), AppError> {
    repository::create_report(
        &state.pool,
        &id,
        &input.name,
        &input.config,
        current_actor(&user).as_deref(),
    )
    .await
    .map(|report| (StatusCode::CREATED, Json(report)))
    .map_err(map_db_error)
}

async fn get_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<repository::Report>, AppError> {
    repository::get_report(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_report(&state.pool, &id)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

async fn list_reports_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::Report>>, AppError> {
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::list_reports(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn get_report_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::Report>, AppError> {
    let report = repository::get_report(&state.pool, &id)
        .await
        .map_err(map_db_error)?;
    repository::check_permission(&state.pool, &report.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    Ok(Json(report))
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

    #[serde(default)]
    pub condition: String,

    #[serde(default)]
    pub required_role: String,
}

async fn create_workflow_transition(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateWorkflowTransition>,
) -> Result<(StatusCode, Json<repository::WorkflowTransition>), AppError> {
    repository::create_workflow_transition_with_options(
        &state.pool,
        &id,
        &input.from_state,
        &input.to_state,
        &input.action,
        &input.condition,
        &input.required_role,
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

async fn get_workflow_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<repository::WorkflowHistoryList>, AppError> {
    repository::list_workflow_history(&state.pool, &id, query.limit, query.offset)
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

#[derive(Debug, Deserialize)]
pub struct ReportAggregateQuery {
    pub entity_id: String,
    pub group_by: String,
}

async fn report_aggregate(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Query(query): Query<ReportAggregateQuery>,
) -> Result<Json<Vec<repository::StatusCount>>, AppError> {
    if query.entity_id.trim().is_empty() || query.group_by.trim().is_empty() {
        return Err(AppError::BadRequest(
            "entity_id and group_by are required".into(),
        ));
    }
    repository::check_permission(&state.pool, &query.entity_id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::report_aggregate_as_role(
        &state.pool,
        &query.entity_id,
        &query.group_by,
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

fn current_role(user: &Option<axum::extract::Extension<auth::User>>) -> String {
    user.as_ref()
        .map(|u| u.role.clone())
        .unwrap_or_else(|| "user".to_string())
}

#[derive(Debug, Deserialize)]
pub struct ExecuteActionRequest {
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub expected_updated_at: Option<String>,
}

async fn execute_module_action(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path((id, action_id)): Path<(String, String)>,
    Json(input): Json<ExecuteActionRequest>,
) -> Result<Json<repository::ModuleActionResult>, AppError> {
    if id.trim().is_empty() || action_id.trim().is_empty() {
        return Err(AppError::BadRequest(
            "entity and action are required".into(),
        ));
    }
    repository::execute_module_action(
        &state.pool,
        &id,
        &action_id,
        input.document_id.as_deref(),
        input.payload.as_ref(),
        current_actor(&user).as_deref(),
        input.expected_updated_at.as_deref(),
        &current_role(&user),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

fn current_actor(user: &Option<axum::extract::Extension<auth::User>>) -> Option<String> {
    user.as_ref().map(|u| u.username.clone())
}

#[derive(Debug, Deserialize)]
pub struct CreateModuleAction {
    pub name: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub config: serde_json::Value,
}

async fn list_module_actions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<repository::ModuleAction>>, AppError> {
    repository::list_module_actions(&state.pool, &id)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn create_module_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<CreateModuleAction>,
) -> Result<(StatusCode, Json<repository::ModuleAction>), AppError> {
    repository::create_module_action(
        &state.pool,
        &id,
        &input.name,
        &input.label,
        &input.kind,
        &input.config,
    )
    .await
    .map(|v| (StatusCode::CREATED, Json(v)))
    .map_err(map_db_error)
}

async fn delete_module_action(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_module_action(&state.pool, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    #[serde(default)]
    pub document_id: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

async fn list_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<repository::EventEntry>>, AppError> {
    repository::list_events(
        &state.pool,
        &id,
        query.document_id.as_deref(),
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
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

async fn create_entity_view_for_user(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<CreateViewRequest>,
) -> Result<(StatusCode, Json<repository::EntityView>), AppError> {
    // Shared views: any role with view access can save the current filter.
    // Delete stays admin-only under /v1/meta/views/{id}.
    repository::check_permission(&state.pool, &id, &current_role(&user), false)
        .await
        .map_err(map_db_error)?;
    repository::create_entity_view(&state.pool, &id, &input.name, &input.config)
        .await
        .map(|view| (StatusCode::CREATED, Json(view)))
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
    repository::delete_document_as_role(
        &state.pool,
        &id,
        current_actor(&user).as_deref(),
        &current_role(&user),
    )
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

async fn list_doc_comments(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<repository::DocCommentList>, AppError> {
    if query.limit < 1 || query.limit > 100 {
        return Err(AppError::BadRequest("limit must be 1..=100".into()));
    }
    if query.offset < 0 {
        return Err(AppError::BadRequest("offset must be >= 0".into()));
    }
    repository::list_doc_comments_as_role(
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
pub struct CreateDocCommentRequest {
    pub body: String,
}

async fn create_doc_comment(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<CreateDocCommentRequest>,
) -> Result<(StatusCode, Json<repository::DocComment>), AppError> {
    repository::create_doc_comment_as_role(
        &state.pool,
        &id,
        &input.body,
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(|comment| (StatusCode::CREATED, Json(comment)))
    .map_err(map_db_error)
}

async fn list_doc_followers(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::DocFollowerList>, AppError> {
    repository::list_doc_followers_as_role(
        &state.pool,
        &id,
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn toggle_doc_follower(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::DocFollowerList>, AppError> {
    repository::toggle_doc_follower_as_role(
        &state.pool,
        &id,
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(Json)
    .map_err(map_db_error)
}

async fn list_doc_activities(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::DocActivityList>, AppError> {
    repository::list_doc_activities_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
pub struct CreateDocActivityRequest {
    pub title: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
}

async fn create_doc_activity(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    Json(input): Json<CreateDocActivityRequest>,
) -> Result<(StatusCode, Json<repository::DocActivity>), AppError> {
    repository::create_doc_activity_as_role(
        &state.pool,
        &id,
        &input.title,
        input.due_date.as_deref(),
        input.assignee.as_deref(),
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(|activity| (StatusCode::CREATED, Json(activity)))
    .map_err(map_db_error)
}

async fn toggle_doc_activity(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::DocActivity>, AppError> {
    repository::toggle_doc_activity_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn list_doc_attachments(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<Json<repository::DocAttachmentList>, AppError> {
    repository::list_doc_attachments_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn upload_doc_attachment(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<(StatusCode, Json<repository::DocAttachment>), AppError> {
    let filename = headers
        .get("x-filename")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();
    let content_type = headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    repository::upload_doc_attachment_as_role(
        &state.pool,
        &id,
        &filename,
        &content_type,
        &body,
        &current_role(&user),
        current_actor(&user).as_deref(),
    )
    .await
    .map(|attachment| (StatusCode::CREATED, Json(attachment)))
    .map_err(map_db_error)
}

async fn download_doc_attachment(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    let data = repository::get_doc_attachment_data_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map_err(map_db_error)?;
    let disposition = format!(
        "attachment; filename=\"{}\"",
        data.filename.replace('"', "")
    );
    Ok((
        [
            ("content-type", data.content_type.clone()),
            ("content-disposition", disposition),
        ],
        data.data,
    )
        .into_response())
}

async fn delete_doc_attachment(
    State(state): State<AppState>,
    user: Option<axum::extract::Extension<auth::User>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    repository::delete_doc_attachment_as_role(&state.pool, &id, &current_role(&user))
        .await
        .map(|()| StatusCode::NO_CONTENT)
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

async fn list_roles(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::rbac::Role>>, AppError> {
    crate::rbac::list_roles(&state.pool)
        .await
        .map(Json)
        .map_err(AppError::from)
}

#[derive(Debug, Deserialize)]
struct CreateRoleRequest {
    name: String,
    label: String,
    #[serde(default)]
    description: String,
}

async fn create_role(
    State(state): State<AppState>,
    Json(input): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<crate::rbac::Role>), AppError> {
    crate::rbac::create_role(&state.pool, &input.name, &input.label, &input.description)
        .await
        .map(|r| (StatusCode::CREATED, Json(r)))
        .map_err(map_db_error)
}

#[derive(Debug, Deserialize)]
struct UpdateRoleRequest {
    label: String,
    #[serde(default)]
    description: String,
}

async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateRoleRequest>,
) -> Result<Json<crate::rbac::Role>, AppError> {
    crate::rbac::update_role(&state.pool, &id, &input.label, &input.description)
        .await
        .map(Json)
        .map_err(map_db_error)
}

async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    crate::rbac::delete_role(&state.pool, &id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_db_error)
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
