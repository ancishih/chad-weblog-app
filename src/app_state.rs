use crate::error::Result;
use axum::extract::FromRef;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use redis::Client as RedisClient;
use reqwest::Client;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub client: Client,
    pub db: PgPool,
    pub oauth_client: BasicClient,
    pub redis: RedisClient,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        dotenv::dotenv().ok();

        let client_id = ClientId::new(
            env::var("GITHUB_CLIENT_ID".to_string()).expect("GITHUB_CLIENT_ID must be set."),
        );

        let client_secret = ClientSecret::new(
            env::var("GITHUB_CLIENT_SECRET")
                .expect("Missing the GITHUB_CLIENT_SECRET env variable."),
        );

        let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .expect("Invalid authorization endpoint URL");

        let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
            .expect("Invalid token endpoint URL");

        let redirect_url = RedirectUrl::new(
            env::var("REDIRECT_URL")
                .unwrap_or_else(|_| "http://localhost:3000/api/auth/github/callback".to_string()),
        );

        let oauth_client =
            BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
                .set_redirect_uri(redirect_url.unwrap());

        let postgres_url = env::var("DATABASE_URL").expect("DATABASE_URL must to be set.");

        let db = PgPoolOptions::new()
            .max_connections(20)
            .connect(&postgres_url)
            .await
            .unwrap();

        let client = Client::new();

        let redis_client = RedisClient::open("redis://127.0.0.1:6379").unwrap();

        Ok(Self {
            client,
            db,
            oauth_client,
            redis: redis_client,
        })
    }
}
