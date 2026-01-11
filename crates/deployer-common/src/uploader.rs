use eyre::Result;
use google_cloud_storage::{
    client::{Client as GcsClient, ClientConfig},
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
    sign::SignedURLOptions,
};
use std::time::Duration;
use std::{env, sync::Arc};
use reqwest::{cookie::Jar, multipart, Url};
use tokio_util::codec::{BytesCodec, FramedRead};
use futures_util::TryStreamExt;

enum UploaderBackend {
    Gcs(google_cloud_storage::client::Client),
    // S3
    Local {
        platform_base: String,
        admin_token: String,
    }
}

pub struct Uploader {
    backend: UploaderBackend,
    bucket: Option<String>,
}

impl Uploader {
    pub async fn from_env() -> Self {
        let mut bucket = std::env::var("GCS_ATTACHMENTS_BUCKET").ok();
        if bucket.is_none() {
            if let Some(alt_bucket) = std::env::var("ATTACHMENTS_BUCKET").ok() {
                bucket = Some(alt_bucket);
            }
        }
        let backend = if std::env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON").is_ok() {
            // build GCS
            UploaderBackend::Gcs(GcsClient::new(
                        ClientConfig::default().with_auth().await.unwrap(),
                    ))
        } else {
            UploaderBackend::Local {
                platform_base: std::env::var("PLATFORM_BASE").unwrap(),
                admin_token: std::env::var("PLATFORM_ADMIN_TOKEN").unwrap(),
            }
        };
        Self {
            backend,
            bucket,
        }

    }

    pub fn get_admin_client(&self) -> Result<reqwest::Client> {
        if let UploaderBackend::Local { platform_base, admin_token } = &self.backend {
            let jar = Jar::default();
            jar.add_cookie_str(
                &format!("admin_token={}", admin_token),
                &Url::parse(&platform_base)?,
            );
            let client = reqwest::Client::builder()
                .cookie_provider(Arc::new(jar))
                .build()?;
            
            Ok(client)
        } else {
            Err(eyre::eyre!("Cannot get admin client for non-local uploader"))
        }
    }

    pub async fn upload(&self, chall_id: &str, filename: &str, data: Vec<u8>) -> Result<String> {
        match &self.backend {
            UploaderBackend::Gcs(gcs_client) => {
                let bucket = self.bucket.as_ref()
                    .ok_or_else(|| eyre::eyre!("No bucket configured for GCS upload"))?;
                
                let upload_type = UploadType::Simple(Media::new(format!("{}/{}", chall_id, filename)));

                let uploaded = gcs_client
                    .upload_object(
                        &UploadObjectRequest {
                            bucket: bucket.clone(),
                            ..Default::default()
                        },
                        data,
                        &upload_type,
                    )
                    .await?;

                let url_for_download = gcs_client
                    .signed_url(
                        bucket,
                        &uploaded.name,
                        None,
                        None,
                        SignedURLOptions {
                            expires: Duration::from_secs(604800),
                            ..Default::default()
                        },
                    )
                    .await?;

                Ok(url_for_download)
            },
            UploaderBackend::Local {
                platform_base,
                admin_token,
            } => {
                let admin_client = self.get_admin_client()?;
                
                let cursor = std::io::Cursor::new(data);
                let stream = FramedRead::new(cursor, BytesCodec::new())
                    .map_ok(|bytes| bytes.freeze());
                
                // submit as multipart form with streaming file upload
                let file_part = multipart::Part::stream(reqwest::Body::wrap_stream(stream))
                    .file_name(filename.to_string());
                
                let form = multipart::Form::new()
                    .part(filename.to_string(), file_part);
                
                // Upload via platform attahment upload API
                let response = admin_client
                    .post(format!("{}/api/admin/attachments/upload?path={}", platform_base, chall_id))
                    .multipart(form)
                    .send()
                    .await?;
                
                if !response.status().is_success() {
                    return Err(eyre::eyre!("Upload failed with status: {}", response.status()));
                }
                
                #[derive(serde::Deserialize)]
                struct UploadResult {
                    url: String,
                }

                // let dbg = response.text().await?;
                // println!("{dbg}");
                
                let results: Vec<UploadResult> = response.json().await?;
                let url = results.first()
                    .ok_or_else(|| eyre::eyre!("No upload result returned"))?
                    .url.clone();
                
                Ok(url)
            }
        }
    }
}