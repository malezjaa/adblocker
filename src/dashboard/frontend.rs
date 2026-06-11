use crate::dashboard::AppError;
use axum::extract::Path;
use axum::http::{Response, header};
use axum::response::IntoResponse;
use include_dir::{Dir, include_dir};
use mime_guess2::from_path;
use reqwest::StatusCode;

pub const DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/dashboard/dist");

pub async fn serve_file(Path(path): Path<String>) -> Result<impl IntoResponse, AppError> {
  let file = if let Some(file) = DIST.get_file(&path) {
    file
  } else if path.contains('.') {
    return Err(AppError::with_code(
      format!("asset {path} not found"),
      StatusCode::NOT_FOUND,
    ));
  } else {
    DIST.get_file("index.html").ok_or_else(|| {
      AppError::with_code("index.html not found", StatusCode::INTERNAL_SERVER_ERROR)
    })?
  };

  let contents = String::from_utf8(file.contents().to_vec())?;

  let mut response = Response::new(contents);

  let mime = if path.contains('.') {
    from_path(&path).first()
  } else {
    from_path("index.html").first()
  };

  if let Some(mime) = mime {
    response.headers_mut().insert(header::CONTENT_TYPE, mime.to_string().parse()?);
  }

  Ok(response)
}
