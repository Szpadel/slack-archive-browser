use std::{fmt::Display, result};

use actix_web::error::ResponseError;
pub use actix_web::http::StatusCode;

#[derive(Debug)]
pub struct WebError {
    err: anyhow::Error,
    code: StatusCode,
}
impl WebError {
    pub fn new(code: StatusCode, msg: &str) -> Self {
        Self {
            code,
            err: anyhow::anyhow!("{}", msg),
        }
    }
}

impl Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#}", self.err))
    }
}

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        self.code
    }
}

impl From<anyhow::Error> for WebError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            err,
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T, E = WebError> = result::Result<T, E>;
