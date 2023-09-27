use serde::{Deserialize, Serialize};
use validator::Validate;
#[derive(Debug, Serialize, Deserialize)]
pub struct Mail {
    pub from: String,
    pub subject: String,
    pub body: String,
}
