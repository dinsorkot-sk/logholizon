use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use crate::{error::AppError, repository, repository::Module};

pub const PACKAGE_SCHEMA_VERSION: u32 = 1;

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
        },
    };
    Ok(serde_json::to_value(package)?)
}

fn parse_package(package: &Value) -> Result<ModulePackage> {
    let package: ModulePackage = serde_json::from_value(package.clone())
        .map_err(|e| AppError::BadRequest(format!("invalid module package: {e}")))?;
    if package.schema_version != PACKAGE_SCHEMA_VERSION {
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
    Ok(package)
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
    crate::module_lifecycle::enable_module(pool, &published.id, owner, "admin").await
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
