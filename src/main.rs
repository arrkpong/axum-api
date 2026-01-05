use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;

async fn index() -> &'static str {
    "Hello, World!"
}

async fn login() -> &'static str {
    "Login endpoint"
}
async fn register() -> &'static str {
    "Register endpoint"
}

async fn logout() -> &'static str {
    "Logout endpoint"
}

async fn admin() -> &'static str {
    "Admin endpoint"
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let _db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let app = Router::new()
        .route("/", get(index))
        .route("/api/v1/login", post(login))
        .route("/api/v1/register", post(register))
        .route("/api/v1/logout", post(logout))
        .route("/api/v1/admin", get(admin));

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    println!("🚀 Server is starting on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
