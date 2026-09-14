use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use crate::{automation, dashboard, error::AppError, relation, repository, repository::Module};

/// Current package schema. Version 1 packages (module definition only) are
/// still accepted; version 2 adds the optional `relations`, `actions`,
/// `automations`, and `dashboards` sections so a package is self-contained.
pub const PACKAGE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePackage {
    pub schema_version: u32,
    pub kind: String,
    pub manifest: ModulePackageManifest,
    pub module: ModulePackageModule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<ModulePackageDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePackageDependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePackageModule {
    pub name: String,
    pub label: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub definition: Value,
    /// Cross-entity relations, authored with definition entity/field names.
    #[serde(default)]
    pub relations: Vec<PackageRelation>,
    /// Record actions, authored per definition entity name.
    #[serde(default)]
    pub actions: Vec<PackageAction>,
    /// Event automations, authored per definition entity name.
    #[serde(default)]
    pub automations: Vec<PackageAutomation>,
    /// Dashboards whose widgets reference definition entity names.
    #[serde(default)]
    pub dashboards: Vec<PackageDashboard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRelation {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub source_field: Option<String>,
    #[serde(default)]
    pub target_field: Option<String>,
    pub name: String,
    pub relation_type: String,
    #[serde(default = "default_relation_on_delete")]
    pub on_delete: String,
}

fn default_relation_on_delete() -> String {
    "restrict".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageAction {
    pub entity: String,
    pub name: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageAutomation {
    pub entity: String,
    pub trigger: String,
    #[serde(default = "default_automation_action")]
    pub action: String,
    pub target_url: String,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub schedule: String,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: i64,
}

fn default_automation_action() -> String {
    "webhook".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_attempts() -> i64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDashboard {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub layout: Value,
    #[serde(default)]
    pub filters: Value,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub users: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackagePreview {
    pub valid: bool,
    pub action: String,
    pub name: String,
    pub version: String,
    pub dependencies: Vec<DependencyCheck>,
    pub conflict: Option<String>,
    pub migration: MigrationPlan,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyCheck {
    pub name: String,
    pub required_version: String,
    pub installed_version: Option<String>,
    pub satisfied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPlan {
    pub creates: Vec<String>,
    pub updates: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn export_package(module: &Module) -> Result<Value> {
    let dependencies = module
        .definition
        .get("dependencies")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|dependency| {
            Some(ModulePackageDependency {
                name: dependency.get("name")?.as_str()?.to_string(),
                version: dependency
                    .get("version")
                    .and_then(Value::as_str)
                    .unwrap_or("*")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();

    let package = ModulePackage {
        schema_version: PACKAGE_SCHEMA_VERSION,
        kind: "logholizon.module".to_string(),
        manifest: ModulePackageManifest {
            name: module.name.clone(),
            version: module.semantic_version.clone(),
            dependencies,
        },
        module: ModulePackageModule {
            name: module.name.clone(),
            label: module.label.clone(),
            description: module.description.clone(),
            icon: module.icon.clone(),
            color: module.color.clone(),
            definition: module.definition.clone(),
            // Export is definition-only: live relations/actions/automations/
            // dashboards are runtime rows, not part of the portable package.
            // Package authors declare them in the v2 sections instead.
            relations: Vec::new(),
            actions: Vec::new(),
            automations: Vec::new(),
            dashboards: Vec::new(),
        },
    };
    Ok(serde_json::to_value(package)?)
}

fn parse_package(package: &Value) -> Result<ModulePackage> {
    let package: ModulePackage = serde_json::from_value(package.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid module package: {e}")))?;
    // Schema 1 (definition only) and 2 (self-contained) are accepted.
    if package.schema_version != 1 && package.schema_version != PACKAGE_SCHEMA_VERSION {
        bail!(
            "unsupported module package schema version: {}",
            package.schema_version
        );
    }
    if package.kind != "logholizon.module" {
        bail!("unsupported module package kind: {}", package.kind);
    }
    if package.manifest.name != package.module.name {
        bail!("package manifest/module name mismatch");
    }
    repository::validate_module_definition(&package.module.definition)?;
    if package.manifest.version.trim().is_empty() {
        bail!("package version is required");
    }
    for dependency in &package.manifest.dependencies {
        if dependency.name.trim().is_empty() || dependency.version.trim().is_empty() {
            bail!("package dependency name and version are required");
        }
    }
    validate_package_sections(&package)?;
    Ok(package)
}

/// Validate the v2 self-contained sections against the declared entities.
/// Names are definition-level (`customer`, not `<module>_customer`); the
/// installer resolves them to materialized IDs after publish.
fn validate_package_sections(package: &ModulePackage) -> Result<()> {
    let entities: std::collections::HashSet<String> = package
        .module
        .definition
        .get("entities")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(|e| e.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    for rel in &package.module.relations {
        if rel.source.trim().is_empty()
            || rel.target.trim().is_empty()
            || rel.name.trim().is_empty()
        {
            bail!("package relation requires source, target, and name");
        }
        for entity in [&rel.source, &rel.target] {
            if !entities.contains(entity) {
                bail!("package relation references unknown entity: {entity}");
            }
        }
        if !matches!(
            rel.relation_type.as_str(),
            "one_to_one" | "one_to_many" | "many_to_one" | "many_to_many"
        ) {
            bail!("invalid package relation type: {}", rel.relation_type);
        }
        if !matches!(
            rel.on_delete.as_str(),
            "restrict" | "cascade" | "set_null"
        ) {
            bail!("invalid package relation on_delete: {}", rel.on_delete);
        }
    }
    for action in &package.module.actions {
        if !entities.contains(&action.entity) {
            bail!("package action references unknown entity: {}", action.entity);
        }
        if action.name.trim().is_empty() || action.label.trim().is_empty() {
            bail!("package action requires name and label");
        }
        if !matches!(
            action.kind.as_str(),
            "create" | "update" | "delete" | "change_status" | "notify" | "webhook" | "formula" | "generate"
        ) {
            bail!("invalid package action kind: {}", action.kind);
        }
    }
    for automation in &package.module.automations {
        if !entities.contains(&automation.entity) {
            bail!(
                "package automation references unknown entity: {}",
                automation.entity
            );
        }
        if !matches!(
            automation.trigger.as_str(),
            "create" | "update" | "delete" | "transition"
        ) {
            bail!("invalid package automation trigger: {}", automation.trigger);
        }
        if !matches!(automation.action.as_str(), "webhook" | "notify") {
            bail!("invalid package automation action: {}", automation.action);
        }
        if automation.target_url.trim().is_empty() {
            bail!("package automation target_url is required");
        }
        crate::security::validate_outbound_url(&automation.target_url)?;
    }
    for dashboard in &package.module.dashboards {
        if dashboard.name.trim().is_empty() {
            bail!("package dashboard requires a name");
        }
        let widgets = dashboard.layout.as_array().cloned().unwrap_or_default();
        for widget in &widgets {
            let kind = widget.get("kind").and_then(Value::as_str).unwrap_or("");
            if !matches!(
                kind,
                "kpi" | "table" | "list" | "bar" | "line" | "pie" | "area"
            ) {
                bail!("invalid package dashboard widget kind: {kind}");
            }
            let entity = widget
                .get("entity_id")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !entities.contains(entity) {
                bail!("package dashboard widget references unknown entity: {entity}");
            }
        }
    }
    Ok(())
}

pub async fn preview_package(
    pool: &SqlitePool,
    package: &Value,
    owner: &str,
) -> Result<PackagePreview> {
    let package = parse_package(package)?;
    let owner = owner.trim();
    let module_id = format!("{}_{}", owner, package.module.name);
    let existing = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT id, semantic_version, version FROM _module WHERE id = ?",
    )
    .bind(&module_id)
    .fetch_optional(pool)
    .await?;

    let mut dependencies = Vec::new();
    for dependency in &package.manifest.dependencies {
        let dep_id = format!("{}_{}", owner, dependency.name);
        let installed = sqlx::query_scalar::<_, String>(
            "SELECT semantic_version FROM _module WHERE id = ? AND status IN ('published','enabled','disabled')",
        )
        .bind(dep_id)
        .fetch_optional(pool)
        .await?;
        dependencies.push(DependencyCheck {
            name: dependency.name.clone(),
            required_version: dependency.version.clone(),
            satisfied: installed.is_some(),
            installed_version: installed,
        });
    }

    let mut creates = Vec::new();
    let mut updates = Vec::new();
    if let Some(entities) = package
        .module
        .definition
        .get("entities")
        .and_then(Value::as_array)
    {
        for entity in entities {
            if let Some(name) = entity.get("name").and_then(Value::as_str) {
                let entity_id = format!("{}_{}", module_id, name);
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _meta_entity WHERE id = ?)")
                        .bind(entity_id)
                        .fetch_one(pool)
                        .await?;
                if exists {
                    updates.push(name.to_string());
                } else {
                    creates.push(name.to_string());
                }
            }
        }
    }
    let migration = MigrationPlan {
        creates,
        updates,
        warnings: Vec::new(),
    };
    let dependency_failure = dependencies
        .iter()
        .find(|d| !d.satisfied)
        .map(|d| format!("missing dependency: {}", d.name));
    let conflict = existing
        .as_ref()
        .map(|(id, version, _)| format!("module already exists: {id} ({version})"));
    Ok(PackagePreview {
        valid: conflict.is_none() && dependency_failure.is_none(),
        action: if existing.is_some() {
            "conflict"
        } else {
            "install"
        }
        .to_string(),
        name: package.module.name,
        version: package.manifest.version,
        dependencies,
        conflict: conflict.or(dependency_failure),
        migration,
    })
}

pub async fn install_package(
    pool: &SqlitePool,
    package: &Value,
    owner: &str,
    actor: Option<&str>,
) -> Result<Module> {
    let package = parse_package(package)?;
    let preview = preview_package(pool, &serde_json::to_value(&package)?, owner).await?;
    if !preview.valid {
        return Err(AppError::Conflict(
            preview
                .conflict
                .unwrap_or_else(|| "package cannot be installed".to_string()),
        )
        .into());
    }
    let module = repository::create_module(
        pool,
        &package.module.name,
        &package.module.label,
        Some(&package.module.description),
        Some(&package.module.icon),
        Some(&package.module.color),
        owner,
        &package.module.definition,
        actor,
    )
    .await?;

    // Imported packages are installed through the normal lifecycle so all
    // validation, compatibility checks, version snapshots, and materialization
    // use the same runtime contracts as user-created modules.

    let reviewed =
        crate::module_lifecycle::submit_module_for_review(pool, &module.id, owner, "admin").await?;
    let published = repository::publish_module(pool, &reviewed.id, owner, "admin", actor).await?;
    let enabled = crate::module_lifecycle::enable_module(pool, &published.id, owner, "admin").await?;
    // Self-contained (v2) sections materialize after publish, when entity
    // and field IDs exist. Names in the package are definition-level and
    // resolve to `<module>_<entity>` / `<entity>_<field>` IDs here.
    materialize_package_sections(pool, &package, &enabled.id, actor).await?;
    repository::get_module(pool, &enabled.id, owner, "admin").await
}

/// Materialize a package's relations, actions, automations, and dashboards.
async fn materialize_package_sections(
    pool: &SqlitePool,
    package: &ModulePackage,
    module_id: &str,
    actor: Option<&str>,
) -> Result<()> {
    let entity_id = |name: &str| format!("{module_id}_{}", name.trim());
    for rel in &package.module.relations {
        let source_id = entity_id(&rel.source);
        let target_id = entity_id(&rel.target);
        let source_field_id = match rel.source_field.as_deref().map(str::trim) {
            Some(field) if !field.is_empty() => Some(resolve_field_id(pool, &source_id, field).await?),
            _ => {
                if rel.relation_type != "many_to_many" {
                    bail!(
                        "package relation {} requires source_field for {}",
                        rel.name,
                        rel.relation_type
                    );
                }
                None
            }
        };
        let target_field_id = match rel.target_field.as_deref().map(str::trim) {
            Some(field) if !field.is_empty() => Some(resolve_field_id(pool, &target_id, field).await?),
            _ => None,
        };
        relation::create_relation(
            pool,
            &source_id,
            source_field_id.as_deref(),
            &target_id,
            target_field_id.as_deref(),
            rel.name.trim(),
            rel.relation_type.trim(),
            rel.on_delete.trim(),
        )
        .await?;
    }
    for action in &package.module.actions {
        repository::create_module_action(
            pool,
            &entity_id(&action.entity),
            action.name.trim(),
            action.label.trim(),
            action.kind.trim(),
            &action.config,
        )
        .await?;
    }
    for item in &package.module.automations {
        let created = repository::create_automation(
            pool,
            &entity_id(&item.entity),
            item.trigger.trim(),
            item.action.trim(),
            item.target_url.trim(),
            item.active,
        )
        .await?;
        automation::update(
            pool,
            &created.id,
            Some(item.condition.as_str()),
            Some(item.schedule.as_str()),
            None,
            Some(item.max_attempts),
            Some(item.active),
        )
        .await?;
    }
    for item in &package.module.dashboards {
        let mut layout = item.layout.clone();
        // Widgets reference definition entity names; rewrite to the
        // materialized IDs the dashboard runtime queries.
        if let Some(widgets) = layout.as_array_mut() {
            for widget in widgets.iter_mut() {
                if let Some(entity) = widget.get("entity_id").and_then(Value::as_str) {
                    let resolved = entity_id(entity);
                    if let Some(object) = widget.as_object_mut() {
                        object.insert(
                            "entity_id".to_string(),
                            Value::String(resolved),
                        );
                    }
                }
            }
        }
        dashboard::create(
            pool,
            item.name.trim(),
            item.description.as_str(),
            &layout,
            &item.filters,
            &item.roles,
            &item.users,
            actor,
        )
        .await?;
    }
    Ok(())
}

/// Resolve a definition-level field name to its materialized field ID.
async fn resolve_field_id(
    pool: &SqlitePool,
    entity_id: &str,
    field_name: &str,
) -> Result<String> {
    let fields = repository::list_fields(pool, entity_id).await?;
    fields
        .iter()
        .find(|f| f.name == field_name)
        .map(|f| f.id.clone())
        .ok_or_else(|| {
            AppError::BadRequest(format!("package references unknown field: {entity_id}.{field_name}")).into()
        })
}

pub async fn uninstall_package(
    pool: &SqlitePool,
    id: &str,
    owner: &str,
    role: &str,
) -> Result<Module> {
    let module = repository::get_module(pool, id, owner, role).await?;
    if module.status == "enabled" {
        return Err(AppError::Conflict("disable module before uninstall".into()).into());
    }
    if module.status == "published" {
        return Err(AppError::Conflict("disable module before uninstall".into()).into());
    }
    if module.status == "archived" {
        return Ok(module);
    }
    repository::archive_module(pool, id, owner, role).await
}
