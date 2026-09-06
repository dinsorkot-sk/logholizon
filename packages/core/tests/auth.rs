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

#[tokio::test]
async fn user_management_crud_and_guards() {
    let pool = db::connect("sqlite::memory:").await.unwrap();
    db::migrate(&pool).await.unwrap();

    assert!(!auth::has_users(&pool).await.unwrap());

    let admin = auth::register(&pool, "admin", "password123").await.unwrap();
    assert!(auth::has_users(&pool).await.unwrap());

    let alice = auth::create_user(&pool, "alice", "password123", "user")
        .await
        .unwrap();
    assert_eq!(alice.role, "user");

    // Invalid role rejected.
    assert!(auth::create_user(&pool, "bob", "password123", "superuser")
        .await
        .is_err());

    // List users ordered by username.
    let users = auth::list_users(&pool).await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "admin");
    assert_eq!(users[1].username, "alice");

    // Promote alice to admin.
    let alice = auth::update_user_role(&pool, &alice.id, "admin")
        .await
        .unwrap();
    assert_eq!(alice.role, "admin");

    // Reset password works.
    auth::reset_password(&pool, &alice.id, "newpassword123")
        .await
        .unwrap();
    auth::login(&pool, "alice", "newpassword123").await.unwrap();
    assert!(auth::login(&pool, "alice", "password123").await.is_err());

    // Deleting one admin is fine while another remains.
    auth::delete_user(&pool, &admin.id).await.unwrap();

    // Cannot delete the last admin.
    let err = auth::delete_user(&pool, &alice.id).await.unwrap_err();
    assert!(err.to_string().contains("last admin"));

    // Unknown user rejected.
    assert!(auth::delete_user(&pool, "user_missing").await.is_err());
    assert!(auth::reset_password(&pool, "user_missing", "password123")
        .await
        .is_err());
}
