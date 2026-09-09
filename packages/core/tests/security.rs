use logholizon_core::security;

#[test]
fn outbound_url_blocks_local_and_private_targets() {
    assert!(security::validate_outbound_url("https://example.com/hook").is_ok());
    assert!(security::validate_outbound_url("http://localhost:8080/hook").is_err());
    assert!(security::validate_outbound_url("http://127.0.0.1:8080/hook").is_err());
    assert!(security::validate_outbound_url("http://10.0.0.5/hook").is_err());
    assert!(security::validate_outbound_url("http://192.168.1.10/hook").is_err());
    assert!(security::validate_outbound_url("file:///etc/passwd").is_err());
}

#[test]
fn outbound_url_rejects_credentials_and_fragments() {
    assert!(security::validate_outbound_url("https://user:pass@example.com/hook").is_err());
    assert!(security::validate_outbound_url("https://example.com/hook#secret").is_err());
}

#[test]
fn webhook_headers_reject_hop_by_hop_headers() {
    assert!(security::validate_webhook_headers(&serde_json::json!({"X-Test":"ok"})).is_ok());
    assert!(security::validate_webhook_headers(&serde_json::json!({"Host":"internal"})).is_err());
    assert!(
        security::validate_webhook_headers(&serde_json::json!({"Content-Length":"1"})).is_err()
    );
}

#[test]
fn filenames_cannot_escape_attachment_namespace() {
    assert!(security::validate_filename("report.pdf").is_ok());
    assert!(security::validate_filename("../secret.txt").is_err());
    assert!(security::validate_filename("..\\secret.txt").is_err());
    assert!(security::validate_filename("evil\nname.txt").is_err());
}

#[tokio::test]
async fn tenant_access_blocks_cross_tenant_module_entity_and_document() {
    let pool = logholizon_core::db::connect("sqlite::memory:")
        .await
        .unwrap();
    logholizon_core::db::migrate(&pool).await.unwrap();
    let definition = serde_json::json!({
        "entities": [{
            "name": "record",
            "label": "Record",
            "fields": [{"name": "name", "type": "text"}]
        }]
    });
    let alice = logholizon_core::repository::create_module(
        &pool,
        "tenant_records",
        "Tenant Records",
        None,
        None,
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    logholizon_core::module_lifecycle::submit_module_for_review(&pool, &alice.id, "alice", "user")
        .await
        .unwrap();
    logholizon_core::repository::publish_module(&pool, &alice.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    let entity_id = format!("{}_record", alice.id);
    assert!(logholizon_core::repository::check_entity_tenant_access(
        &pool, &entity_id, "alice", "user"
    )
    .await
    .is_ok());
    assert!(logholizon_core::repository::check_entity_tenant_access(
        &pool, &entity_id, "bob", "user"
    )
    .await
    .is_err());
    let doc = logholizon_core::repository::create_document_as_role(
        &pool,
        "alice-doc",
        &entity_id,
        &serde_json::json!({"name":"Alice"}),
        Some("alice"),
        "user",
    )
    .await
    .unwrap();
    assert!(logholizon_core::repository::check_document_tenant_access(
        &pool, &doc.id, "alice", "user"
    )
    .await
    .is_ok());
    assert!(logholizon_core::repository::check_document_tenant_access(
        &pool, &doc.id, "bob", "user"
    )
    .await
    .is_err());
    let tenant: String = sqlx::query_scalar("SELECT tenant_id FROM _meta_entity WHERE id = ?")
        .bind(&entity_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tenant, "alice");
    let doc_tenant: String = sqlx::query_scalar("SELECT tenant_id FROM _doc WHERE id = ?")
        .bind(&doc.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_tenant, "alice");
}
