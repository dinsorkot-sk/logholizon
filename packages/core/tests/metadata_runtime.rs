use logholizon_core::{db, relation, repository};
use serde_json::json;

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn metadata_drives_entity_field_relation_and_crud_runtime() {
    let pool = setup().await;
    let definition = json!({
        "entities": [
            {"name":"category","label":"Category","fields":[
                {"name":"code","type":"text","required":true,"unique":true,"min_length":2,"max_length":12},
                {"name":"name","type":"text","required":true}
            ]},
            {"name":"product","label":"Product","fields":[
                {"name":"sku","type":"text","required":true,"unique":true,"pattern":"^SKU-[0-9]+$"},
                {"name":"name","type":"text","required":true},
                {"name":"price","type":"number","required":true,"min_value":0,"max_value":100000},
                {"name":"category","type":"reference","ref_entity":"category"}
            ]}
        ]
    });

    let module = repository::create_module(
        &pool,
        "inventory_runtime",
        "Inventory Runtime",
        None,
        None,
        None,
        "alice",
        &definition,
        Some("alice"),
    )
    .await
    .unwrap();
    repository::submit_module_for_review(&pool, &module.id, "alice", "user")
        .await
        .unwrap();
    let module = repository::publish_module(&pool, &module.id, "alice", "user", Some("alice"))
        .await
        .unwrap();
    assert_eq!(module.status, "published");

    let category = format!("{}_category", module.id);
    let product = format!("{}_product", module.id);
    let fields = repository::list_fields(&pool, &product).await.unwrap();
    let sku = fields.iter().find(|f| f.name == "sku").unwrap();
    assert!(sku.required && sku.is_unique);
    assert_eq!(sku.pattern.as_deref(), Some("^SKU-[0-9]+$"));
    let price = fields.iter().find(|f| f.name == "price").unwrap();
    assert_eq!(price.min_value, Some(0.0));
    assert_eq!(price.max_value, Some(100000.0));

    let relation_field = fields.iter().find(|f| f.name == "category").unwrap();
    assert_eq!(
        relation_field.ref_entity.as_deref(),
        Some(category.as_str())
    );
    let relation = relation::create_relation(
        &pool,
        &product,
        Some(&relation_field.id),
        &category,
        None,
        "product_category",
        "many_to_one",
        "restrict",
    )
    .await
    .unwrap();
    assert_eq!(relation.source_entity_id, product);
    assert_eq!(relation.target_entity_id, category);

    repository::create_document(
        &pool,
        "cat-1",
        &category,
        &json!({"code":"ELEC","name":"Electronics"}),
        Some("alice"),
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "prod-1",
        &product,
        &json!({"sku":"SKU-1001","name":"Keyboard","price":99.5,"category":"cat-1"}),
        Some("alice"),
    )
    .await
    .unwrap();

    // Required, unique, numeric range, pattern, and reference rules are
    // evaluated from metadata rather than from an ERP-specific implementation.
    let err = repository::create_document(
        &pool,
        "prod-missing",
        &product,
        &json!({"sku":"SKU-1002","price":10,"category":"cat-1"}),
        Some("alice"),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("missing required field: name"));

    let err = repository::create_document(
        &pool,
        "prod-duplicate",
        &product,
        &json!({"sku":"SKU-1001","name":"Duplicate","price":10,"category":"cat-1"}),
        Some("alice"),
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("duplicate value for unique field: sku"));

    let err = repository::create_document(
        &pool,
        "prod-range",
        &product,
        &json!({"sku":"SKU-1003","name":"Bad Price","price":-1,"category":"cat-1"}),
        Some("alice"),
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("value below minimum for field price"));

    let err = repository::create_document(
        &pool,
        "prod-pattern",
        &product,
        &json!({"sku":"BAD-1004","name":"Bad SKU","price":10,"category":"cat-1"}),
        Some("alice"),
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("value does not match pattern for field sku"));

    let err = repository::create_document(
        &pool,
        "prod-reference",
        &product,
        &json!({"sku":"SKU-1005","name":"Bad Ref","price":10,"category":"missing-category"}),
        Some("alice"),
    )
    .await
    .unwrap_err();
    assert!(err
        .to_string()
        .contains("unknown reference: category is not"));

    let updated = repository::update_document_as_role(
        &pool,
        "prod-1",
        &json!({"price":129.0}),
        Some("alice"),
        None,
        "user",
    )
    .await
    .unwrap();
    assert_eq!(updated.payload["price"], 129.0);

    let list = repository::list_documents_as_role(
        &pool,
        &product,
        50,
        0,
        &repository::ListDocumentsFilter {
            search: Some("Keyboard".into()),
            ..Default::default()
        },
        "user",
    )
    .await
    .unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].payload["sku"], "SKU-1001");
}
