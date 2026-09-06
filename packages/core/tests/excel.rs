use logholizon_core::{db, repository};
use serde_json::json;

async fn seeded_pool() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "ticket", "ticket", "Ticket")
        .await
        .unwrap();
    repository::create_field(&pool, "ticket", "title", "text", true, false)
        .await
        .unwrap();
    repository::create_field(&pool, "ticket", "priority", "number", false, false)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn export_round_trip_matches_fields() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "d1",
        "ticket",
        &json!({"title": "Fix pump", "priority": 1}),
        None,
    )
    .await
    .unwrap();
    repository::create_document(
        &pool,
        "d2",
        "ticket",
        &json!({"title": "Quote, \"comma\"", "priority": 2}),
        None,
    )
    .await
    .unwrap();
    let csv = repository::export_documents_csv(&pool, "ticket")
        .await
        .unwrap();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines[0], "id,title,priority");
    assert!(lines.contains(&"d1,Fix pump,1"), "{lines:?}");
    assert!(
        lines.contains(&"d2,\"Quote, \"\"comma\"\"\",2"),
        "{lines:?}"
    );
}

#[tokio::test]
async fn export_rejects_over_1000_rows() {
    let pool = seeded_pool().await;
    for index in 0..1001 {
        repository::create_document(
            &pool,
            &format!("d{index}"),
            "ticket",
            &json!({"title": "t"}),
            None,
        )
        .await
        .unwrap();
    }
    let err = repository::export_documents_csv(&pool, "ticket")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("exceeds 1000 rows"), "{err}");
}

#[tokio::test]
async fn preview_reports_row_errors() {
    let pool = seeded_pool().await;
    // required number field: an unparseable value must surface as missing-required
    repository::create_field(&pool, "ticket", "qty", "number", true, false)
        .await
        .unwrap();
    let err = repository::preview_documents_csv(&pool, "ticket", "id,title\nx1,a\n")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("header does not match"), "{err}");

    let csv = "id,title,priority,qty\nx1,a,1,2\nx1,b,2,3\nx2,c,1\nx3,=SUM(A1),1,2\nx4,ok,1,abc\n";
    let preview = repository::preview_documents_csv(&pool, "ticket", csv)
        .await
        .unwrap();
    let errors = preview.errors.join("\n");
    assert!(errors.contains("duplicate id"), "{errors}");
    assert!(errors.contains("wrong column count"), "{errors}");
    assert!(
        errors.contains("formula values are not allowed"),
        "{errors}"
    );
    assert!(errors.contains("missing required field"), "{errors}");
}

#[tokio::test]
async fn confirm_import_creates_and_updates_with_audit() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "x1",
        "ticket",
        &json!({"title": "old", "priority": 1}),
        None,
    )
    .await
    .unwrap();
    let csv = "id,title,priority\nx1,new,2\nx2,added,3\n";
    let result = repository::confirm_documents_csv(&pool, "ticket", csv, None)
        .await
        .unwrap();
    assert_eq!(result.created, 1);
    assert_eq!(result.updated, 1);

    let doc = repository::get_document(&pool, "x1").await.unwrap();
    assert_eq!(doc.payload["title"], "new");
    assert_eq!(doc.payload["priority"], 2.0);

    let audit = repository::list_document_audit(&pool, "x1", 10, 0)
        .await
        .unwrap();
    assert_eq!(audit.items[0].action, "import");
    let audit = repository::list_document_audit(&pool, "x2", 10, 0)
        .await
        .unwrap();
    assert_eq!(audit.items[0].action, "import");
}

#[tokio::test]
async fn confirm_import_rolls_back_on_error() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_entity(&pool, "alpha", "alpha", "Alpha")
        .await
        .unwrap();
    repository::create_field(&pool, "alpha", "title", "text", true, false)
        .await
        .unwrap();
    repository::create_entity(&pool, "beta", "beta", "Beta")
        .await
        .unwrap();
    repository::create_field(&pool, "beta", "title", "text", true, false)
        .await
        .unwrap();
    // x1 belongs to beta
    repository::create_document(
        &pool,
        "x1",
        "beta",
        &json!({"title": "owned by beta"}),
        None,
    )
    .await
    .unwrap();

    let csv = "id,title\nx2,new\nx1,hijack\n";
    let err = repository::confirm_documents_csv(&pool, "alpha", csv, None)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("belongs to another entity"),
        "{err}"
    );

    // x2 must not exist — the whole transaction rolled back
    assert!(repository::get_document(&pool, "x2").await.is_err());
    // x1 unchanged
    let doc = repository::get_document(&pool, "x1").await.unwrap();
    assert_eq!(doc.payload["title"], "owned by beta");
}

#[tokio::test]
async fn negative_number_is_not_formula() {
    let pool = seeded_pool().await;
    let csv = "id,title,priority\nn1,neg,-5\nn2,pos,+5\nn3,bad,=1+1\nn4,bad2,-1+1\n";
    let preview = repository::preview_documents_csv(&pool, "ticket", csv)
        .await
        .unwrap();
    let errors = preview.errors.join("\n");
    assert!(!errors.contains("row 2"), "{errors}");
    assert!(!errors.contains("row 3"), "{errors}");
    assert!(errors.contains("row 4"), "{errors}");
    assert!(errors.contains("row 5"), "{errors}");

    let n1 = preview.rows.iter().find(|row| row["id"] == "n1").unwrap();
    assert_eq!(n1["payload"]["priority"], -5.0);
    let n2 = preview.rows.iter().find(|row| row["id"] == "n2").unwrap();
    assert_eq!(n2["payload"]["priority"], 5.0);
}
