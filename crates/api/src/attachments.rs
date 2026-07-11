use std::path::{Component, Path, PathBuf};

use crate::config::Config;

pub struct AttachmentService {
    pub attachments_path: Option<PathBuf>,
    pub attachments_serving_url: String,
}

impl AttachmentService {
    pub fn new(config: &Config) -> Self {
        let maybe_attachments_dir_string = config.local_attachments_directory.clone();
        let maybe_attachments_serving_url = config.local_attachments_base_serving_url.clone();

        let maybe_attachment_path =
            if let Some(attachments_dir_string) = maybe_attachments_dir_string.as_ref() {
                Some(PathBuf::from(attachments_dir_string))
            } else {
                None
            };

        if let Some(attachment_path) = &maybe_attachment_path {
            if !attachment_path.exists() {
                // TODO: handle errors
                log::info!(
                    "Creating attachment directory {} (because it doesn't exist yet)",
                    attachment_path.display()
                );
                let _ = std::fs::create_dir_all(attachment_path);
            }
        }

        Self {
            attachments_path: maybe_attachment_path,
            attachments_serving_url: maybe_attachments_serving_url
                .unwrap_or_else(|| format!("{}/api/attachments/download", config.cors_origin)),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.attachments_path.is_some()
    }

    fn canonical_attachments_path(&self) -> Option<PathBuf> {
        self.attachments_path
            .as_ref()
            .and_then(|path| std::fs::canonicalize(path).ok())
    }

    fn is_safe_relative_path(path: &Path) -> bool {
        !path.is_absolute()
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    }

    pub fn check_path(&self, path: &str) -> bool {
        let relative_path = Path::new(path);
        if !Self::is_safe_relative_path(relative_path) {
            return false;
        }

        let Some(attachment_path) = self.canonical_attachments_path() else {
            return false;
        };

        let candidate = attachment_path.join(relative_path);
        let mut existing_path = candidate.as_path();
        while !existing_path.exists() {
            let Some(parent) = existing_path.parent() else {
                return false;
            };
            existing_path = parent;
        }

        std::fs::canonicalize(existing_path).is_ok_and(|path| path.starts_with(&attachment_path))
    }

    pub fn check_path_servable(&self, path: &str) -> bool {
        let relative_path = Path::new(path);
        if !Self::is_safe_relative_path(relative_path) {
            return false;
        }

        let Some(attachment_path) = self.canonical_attachments_path() else {
            return false;
        };

        std::fs::canonicalize(attachment_path.join(relative_path))
            .is_ok_and(|path| path.starts_with(&attachment_path))
    }

    pub fn get_attachment_path(&self, path: &str) -> Option<PathBuf> {
        if !self.check_path(path) {
            return None;
        }

        let attachment_path = self.canonical_attachments_path()?;
        let candidate = attachment_path.join(path);
        if candidate.exists() {
            std::fs::canonicalize(candidate)
                .ok()
                .filter(|path| path.starts_with(&attachment_path))
        } else {
            Some(candidate)
        }
    }
}
