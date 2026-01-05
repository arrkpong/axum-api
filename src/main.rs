use axum::{
    routing::{get, post},
    Router,
};

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
    let app = Router::new()
        .route("/", get(index))
        .route("api/v1/login", post(login))
        .route("api/v1/register", post(register))
        .route("api/v1/logout", post(logout))
        .route("api/v1/admin", get(admin));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
