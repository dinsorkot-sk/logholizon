use logholizon_core::{db, dynamic_crud, repository};
use serde_json::json;

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO _module (id,name,label,description,icon,color,owner,status,version,definition) VALUES ('fleet','fleet','Fleet','','','','alice','enabled',1,'{}')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO _meta_entity (id,name,label,description,settings,module,module_id) VALUES ('fleet_vehicle','vehicle','Vehicle','','{}','Fleet','fleet')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO _meta_field (id,entity_id,name,label,type,required,position) VALUES ('fleet_vehicle_plate','fleet_vehicle','plate','Plate','text',0,0)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO _entity_permission (entity_id,role,can_view,can_edit) VALUES ('fleet_vehicle','admin',1,1)")
        .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn resolves_module_entity_and_scopes_bulk_delete() {
    let pool = setup().await;
    let entity = dynamic_crud::resolve_entity(&pool, "fleet", "vehicle")
        .await
        .unwrap();
    assert_eq!(entity.id, "fleet_vehicle");
    repository::create_document(&pool, "v1", &entity.id, &json!({"plate":"A"}), None)
        .await
        .unwrap();
    repository::create_document(&pool, "v2", &entity.id, &json!({"plate":"B"}), None)
        .await
        .unwrap();
    repository::create_entity(&pool, "other", "other", "Other")
        .await
        .unwrap();
    repository::create_document(&pool, "o1", "other", &json!({}), None)
        .await
        .unwrap();
    let deleted = dynamic_crud::bulk_delete(
        &pool,
        &entity.id,
        &["v1".into(), "missing".into()],
        None,
        "admin",
    )
    .await
    .unwrap();
    assert_eq!(deleted, 1);
    assert!(repository::get_document(&pool, "v1").await.is_err());
    assert!(repository::get_document(&pool, "v2").await.is_ok());
    assert!(repository::get_document(&pool, "o1").await.is_ok());
}
