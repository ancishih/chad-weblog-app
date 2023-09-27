// use crate::app_state::AppState;
use crate::error::Error;
use crate::model::mail::Mail;
use axum::{extract::Json, http::StatusCode, response::IntoResponse, routing::post, Router};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn routes() -> Router {
    Router::new().route("/mail", post(sendmail))
}

pub async fn sendmail(Json(ct): Json<Mail>) -> Result<impl IntoResponse, Error> {
    let email = Message::builder()
        .from(ct.from.parse().unwrap())
        .to("<ancishih@gmail.com>".parse().unwrap())
        .subject(ct.subject.as_str())
        .header(ContentType::TEXT_PLAIN)
        .body(String::from(ct.body))
        .unwrap();

    let creds = Credentials::new("ancishih".to_string(), "pflourlwhwxdjryi".to_string());

    let mailer = SmtpTransport::relay("smtp.googlemail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_t) => Ok((StatusCode::ACCEPTED).into_response()),
        Err(e) => panic!("{:?}", e),
    }

    // doueyrtpgrpvpksh
    // pflourlwhwxdjryi
}
