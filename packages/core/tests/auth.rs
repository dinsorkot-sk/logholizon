use logholizon_core::{auth, db};

#[tokio::test]
async fn register_login_me_logout_flow() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();

    // First user becomes admin.
    let admin = auth::register(&pool, "admin", "password123").await.unwrap();
    assert_eq!(admin.role, "admin");

    // Second user becomes regular user.
    let user = auth::register(&pool, "alice", "password123").await.unwrap();
    assert_eq!(user.role, "user");

    // Duplicate username rejected.
    assert!(auth::register(&pool, "admin", "password123").await.is_err());

    // Login works and returns a session.
    let session = auth::login(&pool, "admin", "password123").await.unwrap();
    assert!(!session.token.is_empty());

    // me resolves the session.
    let me = auth::user_for_token(&pool, &session.token).await.unwrap();
    assert_eq!(me.username, "admin");
    assert_eq!(me.role, "admin");

    // Wrong password rejected.
    assert!(auth::login(&pool, "admin", "wrongpass").await.is_err());

    // Logout invalidates the token.
    auth::logout(&pool, &session.token).await.unwrap();
    assert!(auth::user_for_token(&pool, &session.token).await.is_err());
}

#[tokio::test]
async fn short_password_rejected() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();
    assert!(auth::register(&pool, "bob", "short").await.is_err());
}
