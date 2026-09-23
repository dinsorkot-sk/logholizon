use logholizon_core::{db, rbac, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn rbac_role_management_lifecycle() {
    let pool = setup().await;

    // List default roles: system roles (admin, user, manager, operator, viewer) must exist
    let roles = rbac::list_roles(&pool).await.unwrap();
    assert!(roles.iter().any(|r| r.name == "admin" && r.system));
    assert!(roles.iter().any(|r| r.name == "user" && r.system));
    assert!(roles.iter().any(|r| r.name == "manager" && r.system));
    assert!(roles.iter().any(|r| r.name == "operator" && r.system));
    assert!(roles.iter().any(|r| r.name == "viewer" && r.system));

    // Create custom role
    let custom = rbac::create_role(
        &pool,
        "custom_auditor",
        "Custom Auditor",
        "Audits compliance records",
    )
    .await
    .unwrap();
    assert_eq!(custom.name, "custom_auditor");
    assert!(!custom.system);

    // Update custom role
    let updated = rbac::update_role(&pool, &custom.id, "Auditor Lead", "Lead auditor role")
        .await
        .unwrap();
    assert_eq!(updated.label, "Auditor Lead");

    // Cannot modify or delete system roles
    let admin_role = roles.iter().find(|r| r.name == "admin").unwrap();
    assert!(
        rbac::update_role(&pool, &admin_role.id, "Root", "Root user")
            .await
            .is_err()
    );
    assert!(rbac::delete_role(&pool, &admin_role.id).await.is_err());

    // Delete custom role
    rbac::delete_role(&pool, &custom.id).await.unwrap();
    assert!(rbac::get_role(&pool, &custom.id).await.is_err());
}

#[tokio::test]
async fn rbac_entity_and_capability_permissions() {
    let pool = setup().await;

    // Create a generic entity
    let entity = repository::create_entity(&pool, "project_doc", "project_doc", "Project Doc")
        .await
        .unwrap();

    // Get default entity permissions across system roles
    let perms = repository::get_entity_permissions(&pool, &entity.id)
        .await
        .unwrap();
    assert!(perms.len() >= 5);

    // Update entity permissions for viewer (view-only, no export/import/execute/approve)
    let updated_perms = vec![
        repository::EntityPermission {
            role: "viewer".into(),
            can_view: true,
            can_edit: false,
            can_export: false,
            can_import: false,
            can_execute: false,
            can_approve: false,
        },
        repository::EntityPermission {
            role: "operator".into(),
            can_view: true,
            can_edit: true,
            can_export: true,
            can_import: false,
            can_execute: true,
            can_approve: false,
        },
    ];

    let result = repository::update_entity_permissions(&pool, &entity.id, &updated_perms)
        .await
        .unwrap();

    let viewer_perm = result.iter().find(|p| p.role == "viewer").unwrap();
    assert!(viewer_perm.can_view);
    assert!(!viewer_perm.can_edit);
    assert!(!viewer_perm.can_export);
    assert!(!viewer_perm.can_import);

    let operator_perm = result.iter().find(|p| p.role == "operator").unwrap();
    assert!(operator_perm.can_export);
    assert!(!operator_perm.can_import);

    // Check capability enforcement
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "viewer", "export")
            .await
            .is_err()
    );
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "operator", "export")
            .await
            .is_ok()
    );
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "operator", "import")
            .await
            .is_err()
    );
    // Viewer (all capabilities off) is denied execute and approve
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "viewer", "execute")
            .await
            .is_err()
    );
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "viewer", "approve")
            .await
            .is_err()
    );
    // Operator can execute but cannot approve
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "operator", "execute")
            .await
            .is_ok()
    );
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "operator", "approve")
            .await
            .is_err()
    );
    // Admin always bypasses capability restrictions
    assert!(
        repository::check_entity_capability(&pool, &entity.id, "admin", "export")
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn rbac_record_permissions_scoping() {
    let pool = setup().await;

    let entity =
        repository::create_entity(&pool, "expense_claim", "expense_claim", "Expense Claim")
            .await
            .unwrap();

    // Get default record permissions
    let record_perms = repository::get_record_permissions(&pool, &entity.id)
        .await
        .unwrap();
    assert!(!record_perms.is_empty());

    // Update record permission for operator role to scope = 'own'
    let updated = vec![repository::RecordPermission {
        entity_id: entity.id.clone(),
        role: "operator".into(),
        scope: "own".into(),
        owner_field: Some("created_by".into()),
    }];

    let saved = repository::update_record_permissions(&pool, &entity.id, &updated)
        .await
        .unwrap();
    let operator_rec_perm = saved.iter().find(|p| p.role == "operator").unwrap();
    assert_eq!(operator_rec_perm.scope, "own");
    assert_eq!(operator_rec_perm.owner_field.as_deref(), Some("created_by"));

    // Invalid scope is rejected
    let bad_scope = vec![repository::RecordPermission {
        entity_id: entity.id.clone(),
        role: "operator".into(),
        scope: "invalid".into(),
        owner_field: None,
    }];
    assert!(
        repository::update_record_permissions(&pool, &entity.id, &bad_scope)
            .await
            .is_err()
    );

    // Invalid role is rejected
    let bad_role = vec![repository::RecordPermission {
        entity_id: entity.id.clone(),
        role: "ghost".into(),
        scope: "all".into(),
        owner_field: None,
    }];
    assert!(
        repository::update_record_permissions(&pool, &entity.id, &bad_role)
            .await
            .is_err(),
        "setting a record permission for a non-existent role should be rejected"
    );
}

#[tokio::test]
async fn rbac_field_permissions_enforcement() {
    let pool = setup().await;

    let entity = repository::create_entity(
        &pool,
        "employee_record",
        "employee_record",
        "Employee Record",
    )
    .await
    .unwrap();
    let _name_field =
        repository::create_field(&pool, &entity.id, "name", "text", true, false, None, None)
            .await
            .unwrap();
    let salary_field = repository::create_field(
        &pool, &entity.id, "salary", "number", false, false, None, None,
    )
    .await
    .unwrap();

    // Set salary field permission: viewer cannot view, operator can view but not edit
    repository::update_field_permissions(
        &pool,
        &entity.id,
        &[
            (salary_field.id.clone(), "viewer".into(), false, false),
            (salary_field.id.clone(), "operator".into(), true, false),
        ],
    )
    .await
    .unwrap();

    // Field permission map for viewer should hide salary
    let viewer_map = repository::field_permission_map(&pool, &entity.id, "viewer")
        .await
        .unwrap();
    let (salary_viewer_view, _salary_viewer_edit) = viewer_map.get("salary").copied().unwrap();
    assert!(!salary_viewer_view);

    // Field permission map for operator should allow view but deny edit
    let operator_map = repository::field_permission_map(&pool, &entity.id, "operator")
        .await
        .unwrap();
    let (salary_operator_view, salary_operator_edit) = operator_map.get("salary").copied().unwrap();
    assert!(salary_operator_view);
    assert!(!salary_operator_edit);

    // Name field remains viewable by default
    let (name_viewer_view, _name_viewer_edit) = viewer_map.get("name").copied().unwrap();
    assert!(name_viewer_view);
}
