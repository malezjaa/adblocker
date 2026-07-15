use axum::{
  body::Body,
  extract::Path,
  http::{Response, header},
  response::IntoResponse,
};
use include_dir::{Dir, include_dir};
use reqwest::StatusCode;

use crate::dashboard::AppError;

pub const DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/../dashboard/dist");

fn content_type(path: &str) -> Option<&'static str> {
  let (_, extension) = path.rsplit_once('.')?;

  match extension {
    "css" => Some("text/css"),
    "html" => Some("text/html; charset=utf-8"),
    "js" => Some("application/javascript"),
    "svg" => Some("image/svg+xml"),
    "woff2" => Some("font/woff2"),
    _ => None,
  }
}

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

  let mut response = Response::new(Body::from(file.contents().to_vec()));

  let content_type =
    if path.contains('.') { content_type(&path) } else { content_type("index.html") };

  if let Some(content_type) = content_type {
    response.headers_mut().insert(header::CONTENT_TYPE, content_type.parse()?);
  }

  Ok(response)
}

#[cfg(test)]
mod tests {
  use axum::body::to_bytes;
  use include_dir::File;

  use super::*;

  #[test]
  fn recognizes_embedded_asset_content_types() {
    for (path, expected) in [
      ("index.html", "text/html; charset=utf-8"),
      ("assets/app.css", "text/css"),
      ("assets/app.js", "application/javascript"),
      ("assets/logo.svg", "image/svg+xml"),
      ("assets/font.woff2", "font/woff2"),
    ] {
      assert_eq!(content_type(path), Some(expected));
    }
  }

  fn first_font_file(dir: &'static Dir) -> Option<&'static File<'static>> {
    dir
      .files()
      .find(|file| file.path().extension().is_some_and(|extension| extension == "woff2"))
      .or_else(|| dir.dirs().find_map(first_font_file))
  }

  #[tokio::test]
  async fn serves_embedded_font_assets_as_binary() {
    let font = first_font_file(&DIST)
      .expect("dashboard build should include at least one font asset");
    let path = font.path().to_string_lossy().replace('\\', "/");

    let response = match serve_file(Path(path)).await {
      Ok(response) => response.into_response(),
      Err(err) => err.into_response(),
    };

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.headers().get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok()),
      Some("font/woff2")
    );

    let body = to_bytes(response.into_body(), usize::MAX)
      .await
      .expect("font body should be readable");
    assert_eq!(body.as_ref(), font.contents());
  }
}
