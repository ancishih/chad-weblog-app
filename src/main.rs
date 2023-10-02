use crate::routes::{auth, mail, stock};
use axum::{
    http::{header, Method},
    Router,
};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app_state;
mod error;
mod model;
mod pagination;
mod response;
mod routes;
mod routine;
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    // tracing_subscriber::registry().with(
    //     tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "")
    // );

    let mut app = app_state::AppState::new().await.unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 9955));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers([header::CONTENT_TYPE]);

    sqlx::migrate!("./migrations").run(&app.db).await.unwrap();

    tracing::debug!("listening on {}", addr);

    let collect_routes = Router::new()
        .merge(auth::routes(&mut app))
        .merge(mail::routes())
        .merge(stock::routes(&mut app));

    let _ = routine::routine(app.clone()).await.unwrap();

    let routes = Router::new()
        .nest("/api", collect_routes)
        .layer(cors)
        .layer(CookieManagerLayer::new())
        .into_make_service_with_connect_info::<SocketAddr>();
    axum::Server::bind(&addr).serve(routes).await.unwrap();
}
