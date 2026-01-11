use std::path::PathBuf;

use crate::config::Config;

pub struct AttachmentService {
    pub attachments_path: Option<PathBuf>,
    pub attachments_serving_url: String,
}

impl AttachmentService {
    pub fn new(config: &Config) -> Self {
        let maybe_attachments_dir_string = config.local_attachments_directory.clone();
        let maybe_attachments_serving_url = config.local_attachments_base_serving_url.clone();

        let maybe_attachment_path = if let Some(attachments_dir_string) = maybe_attachments_dir_string.as_ref() {
            Some(PathBuf::from(attachments_dir_string))
        } else {
            None
        };

        if let Some(attachment_path) = &maybe_attachment_path {
            if !attachment_path.exists() {
                // TODO: handle errors
                log::info!("Creating attachment directory {} (because it doesn't exist yet)", attachment_path.display());
                let _ = std::fs::create_dir_all(attachment_path);
            }
        }
        
        Self {
            attachments_path: maybe_attachment_path,
            attachments_serving_url: maybe_attachments_serving_url.unwrap_or_else(|| format!("{}/api/attachments/download", config.cors_origin)),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.attachments_path.is_some()
    }

    pub fn check_path(&self, path: &str) -> bool {
        if let Some(attachment_path) = &self.attachments_path {
            attachment_path.join(path).starts_with(attachment_path)
        } else {
            false
        }
    }

    pub fn check_path_servable(&self, path: &str) -> bool {
        if !self.check_path(path) {
            return false;
        }
        if let Some(attachment_path) = &self.attachments_path {
            attachment_path.join(path).exists()
        } else {
            false
        }
    }

    pub fn get_attachment_path(&self, path: &str) -> Option<PathBuf> {
        if let Some(attachment_path) = &self.attachments_path {
            Some(attachment_path.join(path))
        } else {
            None
        }
    }
}