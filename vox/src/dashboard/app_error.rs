use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;

pub struct AppError(String, Option<StatusCode>);

impl AppError {
  pub fn new(error: String) -> Self {
    Self(error, None)
  }

  pub fn with_code(error: impl Into<String>, code: StatusCode) -> Self {
    Self(error.into(), Some(code))
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let code = self.1.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (code, Json(json!({ "error": self.0, "code": code.as_u16() }))).into_response()
  }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
  fn from(e: E) -> Self {
    Self(e.into().to_string(), None)
  }
}

#[macro_export]
macro_rules! app_error {
    ($($arg:tt)*) => {
      return Err(AppError::new(format!($($arg)*,)))
    };
}
