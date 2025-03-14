mod endpoints;

use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use dotenv::dotenv;
use endpoints::{
    EndpointModule,
    updates::{UPDATES_BASE_ENDPOINT, UpdatesModule},
};
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .route("/", get(root))
        .nest(UPDATES_BASE_ENDPOINT, UpdatesModule::create_router());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn root() -> Json<Value> {
    Json(json!({ "ok": true, "readme": "todo" }))
}
