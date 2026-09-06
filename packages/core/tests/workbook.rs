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
    repository::create_entity(&pool, "asset", "asset", "Asset")
        .await
        .unwrap();
    repository::create_field(&pool, "asset", "name", "text", true, false)
        .await
        .unwrap();
    pool
}

fn sheet_rows(bytes: &[u8], name: &str) -> Vec<Vec<String>> {
    use calamine::Reader;
    let mut workbook: calamine::Xlsx<_> =
        calamine::open_workbook_from_rs(std::io::Cursor::new(bytes)).unwrap();
    workbook
        .worksheet_range(name)
        .unwrap()
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| match cell {
                    calamine::Data::Empty => String::new(),
                    calamine::Data::String(text) => text.clone(),
                    other => other.to_string(),
                })
                .collect()
        })
        .collect()
}

#[tokio::test]
async fn workbook_export_has_one_sheet_per_entity() {
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
    repository::create_document(&pool, "a1", "asset", &json!({"name": "Pump A"}), None)
        .await
        .unwrap();

    let bytes = repository::export_workbook_xlsx(&pool, "admin")
        .await
        .unwrap();
    assert!(!bytes.is_empty());
    // ZIP magic: xlsx is a zip archive.
    assert_eq!(&bytes[0..2], b"PK");

    use calamine::Reader;
    let reader: calamine::Xlsx<_> =
        calamine::open_workbook_from_rs(std::io::Cursor::new(&bytes)).unwrap();
    let mut names = reader.sheet_names();
    names.sort();
    assert_eq!(names, vec!["asset".to_string(), "ticket".to_string()]);

    let ticket = sheet_rows(&bytes, "ticket");
    assert_eq!(ticket[0], vec!["id", "title", "priority"]);
    assert!(ticket
        .iter()
        .any(|row| row[0] == "d1" && row[1] == "Fix pump"));
    let asset = sheet_rows(&bytes, "asset");
    assert_eq!(asset[0], vec!["id", "name"]);
    assert!(asset.iter().any(|row| row[0] == "a1" && row[1] == "Pump A"));
}

#[tokio::test]
async fn workbook_export_rejects_over_1000_rows() {
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
    let err = repository::export_workbook_xlsx(&pool, "admin")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("exceeds 1000 rows"), "{err}");
}

#[tokio::test]
async fn workbook_round_trip_preserves_data() {
    let pool = seeded_pool().await;
    repository::create_document(
        &pool,
        "d1",
        "ticket",
        &json!({"title": "Fix pump", "priority": 2}),
        None,
    )
    .await
    .unwrap();
    repository::create_document(&pool, "a1", "asset", &json!({"name": "Pump A"}), None)
        .await
        .unwrap();

    let bytes = repository::export_workbook_xlsx(&pool, "admin")
        .await
        .unwrap();
    let result = repository::confirm_workbook_xlsx(&pool, &bytes, Some("tester"), "admin")
        .await
        .unwrap();
    assert_eq!(result.sheets.len(), 2);
    let ticket = result
        .sheets
        .iter()
        .find(|sheet| sheet.entity_id == "ticket")
        .unwrap();
    assert_eq!(ticket.updated, 1);
    assert_eq!(ticket.created, 0);

    let doc = repository::get_document(&pool, "d1").await.unwrap();
    assert_eq!(doc.payload["title"], "Fix pump");
    assert_eq!(doc.payload["priority"], 2.0);
    let audit = repository::list_document_audit(&pool, "d1", 10, 0)
        .await
        .unwrap();
    assert_eq!(audit.items[0].action, "import");
    assert_eq!(audit.items[0].actor.as_deref(), Some("tester"));
}

#[tokio::test]
async fn workbook_preview_reports_unknown_sheet_and_row_errors() {
    let pool = seeded_pool().await;
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("nope").unwrap();
    sheet.write_string(0, 0, "id").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();
    let err = repository::preview_workbook_xlsx(&pool, &bytes, "admin")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown entity"), "{err}");

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("ticket").unwrap();
    // Required number field: an unparseable value surfaces as missing-required
    // (empty text cells parse as "" which passes the required check, same as CSV).
    repository::create_field(&pool, "ticket", "qty", "number", true, false)
        .await
        .unwrap();
    sheet.write_string(0, 0, "id").unwrap();
    sheet.write_string(0, 1, "title").unwrap();
    sheet.write_string(0, 2, "priority").unwrap();
    sheet.write_string(0, 3, "qty").unwrap();
    // Duplicate id + unparseable required number on row 4.
    sheet.write_string(1, 0, "x1").unwrap();
    sheet.write_string(1, 1, "a").unwrap();
    sheet.write_number(1, 2, 1.0).unwrap();
    sheet.write_number(1, 3, 2.0).unwrap();
    sheet.write_string(2, 0, "x1").unwrap();
    sheet.write_string(2, 1, "b").unwrap();
    sheet.write_number(2, 2, 2.0).unwrap();
    sheet.write_number(2, 3, 3.0).unwrap();
    sheet.write_string(3, 0, "x2").unwrap();
    sheet.write_string(3, 1, "c").unwrap();
    sheet.write_number(3, 2, 3.0).unwrap();
    sheet.write_string(3, 3, "abc").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();
    let preview = repository::preview_workbook_xlsx(&pool, &bytes, "admin")
        .await
        .unwrap();
    assert_eq!(preview.sheets.len(), 1);
    let errors = preview.sheets[0].errors.join("\n");
    assert!(errors.contains("duplicate id"), "{errors}");
    assert!(errors.contains("missing required field"), "{errors}");
}

#[tokio::test]
async fn workbook_confirm_is_atomic_across_sheets() {
    let pool = seeded_pool().await;
    // x1 belongs to asset; the ticket sheet tries to hijack it.
    repository::create_document(
        &pool,
        "x1",
        "asset",
        &json!({"name": "owned by asset"}),
        None,
    )
    .await
    .unwrap();

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let ticket = workbook.add_worksheet();
    ticket.set_name("ticket").unwrap();
    ticket.write_string(0, 0, "id").unwrap();
    ticket.write_string(0, 1, "title").unwrap();
    ticket.write_string(0, 2, "priority").unwrap();
    ticket.write_string(1, 0, "fresh").unwrap();
    ticket.write_string(1, 1, "new").unwrap();
    ticket.write_number(1, 2, 1.0).unwrap();
    ticket.write_string(2, 0, "x1").unwrap();
    ticket.write_string(2, 1, "hijack").unwrap();
    let asset = workbook.add_worksheet();
    asset.set_name("asset").unwrap();
    asset.write_string(0, 0, "id").unwrap();
    asset.write_string(0, 1, "name").unwrap();
    asset.write_string(1, 0, "a2").unwrap();
    asset.write_string(1, 1, "new asset").unwrap();
    let bytes = workbook.save_to_buffer().unwrap();

    let err = repository::confirm_workbook_xlsx(&pool, &bytes, None, "admin")
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("belongs to another entity"),
        "{err}"
    );
    // Nothing was imported: the whole workbook rolled back.
    assert!(repository::get_document(&pool, "fresh").await.is_err());
    assert!(repository::get_document(&pool, "a2").await.is_err());
    let doc = repository::get_document(&pool, "x1").await.unwrap();
    assert_eq!(doc.payload["name"], "owned by asset");
}

#[tokio::test]
async fn workbook_import_enforces_edit_permission() {
    let pool = seeded_pool().await;
    repository::update_entity_permissions(&pool, "ticket", &[("user".to_string(), true, false)])
        .await
        .unwrap();
    let bytes = repository::export_workbook_xlsx(&pool, "admin")
        .await
        .unwrap();
    let err = repository::preview_workbook_xlsx(&pool, &bytes, "user")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no edit access"), "{err}");
}
