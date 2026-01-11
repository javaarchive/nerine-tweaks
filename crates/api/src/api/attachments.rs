use axum::{
    Json, Router, body::{Body, BodyDataStream}, extract::{Path, State as StateE}, http::{HeaderMap, HeaderValue, StatusCode}, routing::{get, post}
};
use chrono::format;
use lettre::message::header;
use tokio_util::io::ReaderStream;

use crate::{
    extractors::Auth,
    Result, State,
};

async fn download_attachment(
    StateE(state): StateE<State>,
    Path(path): Path<String>,
) -> Result<(StatusCode, HeaderMap, Body)> {
    if !state.attachment_service.is_enabled() {
        log::warn!("Blocked user attachment download because local attachment service is disabled");
        // if you're a well behaved client, you shouldn't get here
        return Err(crate::error::Error::GenericError);
    }
    let mut header_map = HeaderMap::new();
    let rel_path = path.trim_start_matches("/download/");
    if state.attachment_service.check_path_servable(rel_path) {
        let abs_path = state.attachment_service.get_attachment_path(rel_path).unwrap();
        let file = tokio::fs::File::open(&abs_path).await.map_err(|_| crate::error::Error::ServerMisconfiguration)?;
        let stream = ReaderStream::new(file);
        // write content disposition header
        let content_disposition_value = format!("attachment; filename=\"{}\"", (&abs_path).file_name().unwrap_or_default().to_string_lossy());
        header_map.append("Content-Disposition", HeaderValue::from_str(&content_disposition_value).unwrap());
        return Ok((StatusCode::OK, header_map, Body::from_stream(stream)));
    } else {
        return Err(crate::error::Error::GenericError); // TODO: not found error
    }
}

pub fn router() -> Router<State> {
    Router::new()
        .route("/download/{*path}", get(download_attachment))
        
}
