use crate::app_state::AppState;
use crate::error::Error;
use crate::model::user;
use crate::routes::WEBLOG_ID;
use axum::{
    extract::{Query, State},
    headers::{HeaderMap, HeaderValue},
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::{prelude::*, Duration, NaiveDate};
use oauth2::{
    reqwest::async_http_client, AuthorizationCode, CsrfToken, PkceCodeChallenge, TokenResponse,
};
use reqwest::header::{ACCEPT, USER_AGENT};
use time::{Duration as TimeDuration, OffsetDateTime};
use tower_cookies::{
    cookie::{CookieBuilder, SameSite},
    Cookie as TowerCookie, Cookies, Key,
};
use uuid::Uuid;
pub fn routes(app: &mut AppState) -> Router {
    Router::new()
        .route("/auth/github", get(login_with_github))
        .route("/auth/github/callback", get(github_callback))
        .with_state(app.clone())
}

pub async fn login_with_github(State(app): State<AppState>) -> impl IntoResponse {
    let client = app.oauth_client;

    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, _) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    Redirect::to(authorize_url.as_ref())
}

#[derive(Debug, serde::Deserialize)]
struct AuthRequest {
    code: String,
    state: CsrfToken,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct User {
    usr_id: i32,
    username: String,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct SessionId {
    ss_id: Uuid,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
struct ExpireTime {
    expires: NaiveDateTime,
}

async fn github_callback(
    cookie: Cookies,
    State(app): State<AppState>,
    Query(query): Query<AuthRequest>,
) -> Result<impl IntoResponse, Error> {
    let mut headers = HeaderMap::new();

    let client = app.oauth_client;

    let token = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .unwrap();

    let access_token: Result<&String, std::convert::Infallible> =
        token.access_token().secret().try_into();

    let reqwest = app
        .client
        .get("https://api.github.com/user")
        .header(ACCEPT, "application/json")
        .bearer_auth(access_token.unwrap().as_str())
        .header(USER_AGENT, "Rust");

    let res = reqwest
        .send()
        .await
        .unwrap()
        .json::<user::GithubUser>()
        .await?;

    let is_registered = sqlx::query_as::<_, User>(
        r#"SELECT usr_id, username FROM weblog.user WHERE username = ($1)"#,
    )
    .bind(&res.login)
    .fetch_optional(&app.db)
    .await?;

    let local = Local::now();

    let datetime = NaiveDate::from_ymd_opt(local.year(), local.month(), local.day())
        .unwrap()
        .and_hms_opt(local.hour(), local.minute(), local.second())
        .unwrap()
        + Duration::hours(7);

    let mut transaction = app.db.begin().await?;

    match is_registered {
        Some(user) => {
            let ss_id = cookie.get(WEBLOG_ID);

            if ss_id.is_none() {
                let session_id = sqlx::query_as::<_, SessionId>(
                    r#"
                INSERT INTO weblog.session_table(expires, session) VALUES ($1, $2) RETURNING ss_id
                "#,
                )
                .bind(&datetime)
                .bind(&access_token.unwrap().as_str())
                .fetch_one(&mut transaction)
                .await?;

                sqlx::query(r#"UPDATE weblog.who_is_login SET ss_id = ($1) WHERE usr_id = ($2)"#)
                    .bind(&session_id.ss_id.to_string())
                    .bind(&user.usr_id)
                    .execute(&mut transaction)
                    .await?;

                let expiry = OffsetDateTime::now_utc().checked_add(TimeDuration::hours(7));

                let ck = CookieBuilder::new(WEBLOG_ID, session_id.ss_id.to_string())
                    .expires(expiry)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .finish();

                cookie.add(ck);
            }

            headers.insert(header::LOCATION, HeaderValue::from_static("/"));

            Ok((StatusCode::FOUND, headers).into_response())
        }
        None => {
            let session_id = sqlx::query_as::<_,SessionId>(
                r#"with ss as (INSERT INTO weblog.session_table(expires, session) VALUES ($1, $2) RETURNING ss_id)
            ,usr as (INSERT INTO weblog.user(github_id, username, created_at, updated_at) VALUES ($3, $4, current_timestamp, current_timestamp) RETURNING usr_id)
            INSERT INTO weblog.who_is_login(usr_id, ss_id) SELECT usr_id, ss_id FROM ss, usr RETURNING ss_id
            "#,
            ).bind(&datetime)
            .bind(&access_token.unwrap().as_str())
            .bind(&res.id)
            .bind(&res.login)
            .fetch_one( &mut transaction).await.unwrap();

            let expiry = OffsetDateTime::now_utc().checked_add(TimeDuration::hours(7));

            let ck = CookieBuilder::new(WEBLOG_ID, session_id.ss_id.to_string())
                .expires(expiry)
                .http_only(true)
                .same_site(SameSite::Strict)
                .finish();

            cookie.add(ck);

            headers.insert(header::LOCATION, HeaderValue::from_static("/"));

            transaction.commit().await?;

            Ok((StatusCode::FOUND, headers).into_response())
        }
    }
}
