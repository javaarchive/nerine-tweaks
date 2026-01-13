use axum::{
    Json, Router, extract::{DefaultBodyLimit, Multipart, Query, State as StateE}, http::StatusCode, routing::{get, post}
};
use serde::{Deserialize, Serialize};
use tokio::stream;
use tokio_util::io::StreamReader;
use futures_util::{Stream, TryStreamExt};

use crate::{
    Result, State, extractors::{Admin, Auth}
};

#[derive(Deserialize)]
struct UploadParams {
    pub path: Option<String>,
}

#[derive(Serialize)]
struct UploadResult {
    pub url: String,
}

async fn upload_attachment(
    StateE(state): StateE<State>,
    _: Admin,
    params: Query<UploadParams>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<UploadResult>>)> {
    // TODO: yell at admin accurately if attachment serving is disabled.
    if !state.attachment_service.is_enabled() {
        log::warn!("Blocked admin attachment upload because local attachment service is disabled");
        return Err(crate::error::Error::ServerMisconfiguration);
    }
    let dest_rel_path = params.path.clone().unwrap_or_else(|| ".".to_string());
    let mut results = Vec::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if let Some(upload_abs_path) = state.attachment_service.get_attachment_path(&format!("{dest_rel_path}/{name}")) {
            if let Some(parent) = upload_abs_path.parent() {
                if !parent.exists() {
                    // I hate this line, hopefully this never gets triggered
                    // if so it's prob cause someone made their upload dir read only or no space left on device
                    tokio::fs::create_dir_all(parent).await.map_err(|_| crate::error::Error::ServerMisconfiguration)?;
                }
            }
            let file = tokio::fs::File::create(&upload_abs_path).await.map_err(|_| crate::error::Error::ServerMisconfiguration)?;
            let stream = field.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)).into_stream();
            let mut reader = StreamReader::new(stream);
            let mut writer = tokio::io::BufWriter::new(file);

            tokio::io::copy(&mut reader, &mut writer).await.map_err(|_| crate::error::Error::ServerMisconfiguration)?;
            results.push(UploadResult {
                url: state.attachment_service.attachments_serving_url.clone() + &format!("/{dest_rel_path}/{name}"),
            });
        } else {
            log::warn!("Attachment upload failed to resolve path: {:?}", field);
        }
    }
    Ok((StatusCode::OK, Json(results)))
}

pub fn router() -> Router<State> {
    Router::new()
        .route("/upload", post(upload_attachment))
        .layer(DefaultBodyLimit::disable()) // please upload sane sized attachments. this an admin endpoint so we don't enforce limits
}
