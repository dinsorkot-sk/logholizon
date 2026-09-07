use logholizon_core::{db, repository};

async fn setup() -> sqlx::SqlitePool {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    repository::create_currency(&pool, "THB", "Thai Baht", 2)
        .await
        .unwrap();
    pool
}

async fn company(pool: &sqlx::SqlitePool) -> String {
    repository::create_company(pool, "Acme Co.", "THB")
        .await
        .unwrap()
        .id
}

#[tokio::test]
async fn m3_balanced_post_and_trial_balance() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let cash = repository::create_gl_account(&pool, &company_id, "1000", "Cash", "asset")
        .await
        .unwrap();
    let revenue = repository::create_gl_account(&pool, &company_id, "4000", "Revenue", "income")
        .await
        .unwrap();
    let entry = repository::post_journal_entry(
        &pool,
        &company_id,
        "Sale",
        "2026-09-01",
        &[
            repository::JournalLineInput {
                account_id: cash.id.clone(),
                debit: 10700,
                credit: 0,
                memo: String::new(),
            },
            repository::JournalLineInput {
                account_id: revenue.id.clone(),
                debit: 0,
                credit: 10700,
                memo: String::new(),
            },
        ],
        Some("admin"),
    )
    .await
    .unwrap();
    assert_eq!(entry.lines.len(), 2);

    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!(trial.total_debit, 10700);
    assert_eq!(trial.total_credit, 10700);
    let cash_row = trial.rows.iter().find(|row| row.code == "1000").unwrap();
    assert_eq!((cash_row.debit, cash_row.credit), (10700, 0));
}

#[tokio::test]
async fn m3_unbalanced_post_rejected() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let cash = repository::create_gl_account(&pool, &company_id, "1000", "Cash", "asset")
        .await
        .unwrap();
    let revenue = repository::create_gl_account(&pool, &company_id, "4000", "Revenue", "income")
        .await
        .unwrap();
    let err = repository::post_journal_entry(
        &pool,
        &company_id,
        "Bad",
        "2026-09-01",
        &[
            repository::JournalLineInput {
                account_id: cash.id,
                debit: 100,
                credit: 0,
                memo: String::new(),
            },
            repository::JournalLineInput {
                account_id: revenue.id,
                debit: 0,
                credit: 90,
                memo: String::new(),
            },
        ],
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("debits must equal credits"));
}

#[tokio::test]
async fn m3_lock_blocks_post_and_void() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let cash = repository::create_gl_account(&pool, &company_id, "1000", "Cash", "asset")
        .await
        .unwrap();
    let revenue = repository::create_gl_account(&pool, &company_id, "4000", "Revenue", "income")
        .await
        .unwrap();
    let entry = repository::post_journal_entry(
        &pool,
        &company_id,
        "Sale",
        "2026-09-01",
        &[
            repository::JournalLineInput {
                account_id: cash.id.clone(),
                debit: 100,
                credit: 0,
                memo: String::new(),
            },
            repository::JournalLineInput {
                account_id: revenue.id,
                debit: 0,
                credit: 100,
                memo: String::new(),
            },
        ],
        None,
    )
    .await
    .unwrap();
    repository::lock_period(&pool, &company_id, "2026-09", Some("admin"))
        .await
        .unwrap();

    // Posting into a locked period is rejected.
    let err = repository::post_journal_entry(
        &pool,
        &company_id,
        "Late",
        "2026-09-15",
        &[
            repository::JournalLineInput {
                account_id: cash.id,
                debit: 10,
                credit: 0,
                memo: String::new(),
            },
            repository::JournalLineInput {
                account_id: entry.lines[1].account_id.clone(),
                debit: 0,
                credit: 10,
                memo: String::new(),
            },
        ],
        None,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("period locked"));

    // Voiding a locked entry is rejected; history is preserved.
    let err = repository::void_journal_entry(&pool, &entry.id)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("period locked"));
}

#[tokio::test]
async fn m3_void_reverses_without_deleting() {
    let pool = setup().await;
    let company_id = company(&pool).await;
    let cash = repository::create_gl_account(&pool, &company_id, "1000", "Cash", "asset")
        .await
        .unwrap();
    let revenue = repository::create_gl_account(&pool, &company_id, "4000", "Revenue", "income")
        .await
        .unwrap();
    let entry = repository::post_journal_entry(
        &pool,
        &company_id,
        "Sale",
        "2026-09-01",
        &[
            repository::JournalLineInput {
                account_id: cash.id,
                debit: 500,
                credit: 0,
                memo: String::new(),
            },
            repository::JournalLineInput {
                account_id: revenue.id,
                debit: 0,
                credit: 500,
                memo: String::new(),
            },
        ],
        None,
    )
    .await
    .unwrap();
    let voided = repository::void_journal_entry(&pool, &entry.id)
        .await
        .unwrap();
    assert_eq!(voided.status, "void");
    let trial = repository::trial_balance(&pool, &company_id).await.unwrap();
    assert_eq!((trial.total_debit, trial.total_credit), (0, 0));
}
