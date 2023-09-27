use argon2::{self, Config};
use password_hash::SaltString;
use rand::rngs::OsRng;
use serde::Deserialize;
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    usr_id: i32,
    github_id: Option<i64>,
    username: String,
    password_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserFC {
    pub username: String,
    pub password_hash: String,
}

pub fn encoding(code: &str) -> Result<String, argon2::Error> {
    let config = Config::default();

    let salt = SaltString::generate(&mut OsRng);

    argon2::hash_encoded(code.as_bytes(), salt.to_string().as_bytes(), &config)
}

pub fn verify(hashed: String, password: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(&hashed, password.as_bytes())
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: i64,
    node_id: String,
    avatar_url: String,
    gravatar_id: Option<String>,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    type_: Option<String>,
    site_admin: bool,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    pub email: Option<String>,
    hireable: Option<String>,
    bio: Option<String>,
    twitter_username: Option<String>,
    public_repos: i64,
    public_gists: i64,
    followers: i64,
    following: i64,
    created_at: Option<String>,
    updated_at: Option<String>,
}
