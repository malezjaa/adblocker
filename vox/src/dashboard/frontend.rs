use crate::dashboard::AppError;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{Response, header};
use axum::response::IntoResponse;
use include_dir::{Dir, include_dir};
use mime_guess2::from_path;
use reqwest::StatusCode;

pub const DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/../dashboard/dist");

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

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::to_bytes;
  use include_dir::File;

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
