use crate::dashboard::AppError;
use axum::extract::Path;
use axum::http::{Response, header};
use axum::response::IntoResponse;
use include_dir::{Dir, include_dir};
use mime_guess2::from_path;
use reqwest::StatusCode;

pub const DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/dashboard/dist");

pub async fn serve_file(Path(path): Path<String>) -> Result<impl IntoResponse, AppError> {
  let file = DIST.get_file(&path);

  if let Some(file) = file {
    let mut response = Response::new(String::from_utf8(file.contents().to_vec())?);
    let ty = from_path(path);

    if let Some(ty) = ty.first() {
      response.headers_mut().insert(header::CONTENT_TYPE, ty.to_string().parse()?);
    }

    Ok(response)
  } else {
    Err(AppError::with_code(format!("file {path} not found"), StatusCode::NOT_FOUND))
  }
}
