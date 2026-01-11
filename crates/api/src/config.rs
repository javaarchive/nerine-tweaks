use std::{str::FromStr, sync::Arc};

use envconfig::Envconfig;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::{DB, attachments, email, event::Event};

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl FromStr for JwtKeys {
    type Err = jsonwebtoken::errors::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            encoding: EncodingKey::from_base64_secret(s)?,
            decoding: DecodingKey::from_base64_secret(s)?,
        })
    }
}

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,

    #[envconfig(from = "JWT_SECRET")]
    pub jwt_keys: JwtKeys,

    #[envconfig(from = "ADMIN_TOKEN")]
    pub admin_token: String,

    #[envconfig(from = "EVENT_PATH", default = "event.toml")]
    pub event_path: String,

    #[envconfig(from = "CORS_ORIGIN", default = "http://nerine.localhost")]
    pub cors_origin: String,

    #[envconfig(from = "SMTP_URL", default = "")]
    pub smtp_url: String,

    #[envconfig(from = "FROM_EMAIL", default = "noreply@nerine.localhost")]
    pub from_email: String,

    #[envconfig(from = "DEPLOYER_BASE", default = "http://deployer:3001")]
    pub deployer_base: String,

    #[envconfig(from = "BLOODBOT_DISCORD_WEBHOOK_URL")]
    pub bloodbot_discord_webhook_url: Option<String>,

    #[envconfig(from = "INSTANCE_LIFETIME", default = "600")]
    pub instance_lifetime: u64,

    // enabling this will enable api endpoints for local attachment serving
    // the cli needs to be aware of this    
    // for docker it should be /attachments
    #[envconfig(from = "LOCAL_ATTACHMENTS_DIRECTORY")]
    pub local_attachments_directory: Option<String>,

    // allow another server (e.g. caddy serve static) to serve attachments
    #[envconfig(from = "LOCAL_ATTACHMENTS_BASE_SERVING_URL")]
    pub local_attachments_base_serving_url: Option<String>,

    #[envconfig(from = "EMAIL_DOMAIN_WHITELIST")]
    pub email_domain_whitelist: Option<String>,
}

pub struct StateInner {
    pub config: Config,
    pub event: Event,
    pub db: DB,
    pub email: email::EmailService,
    pub attachment_service: attachments::AttachmentService,
}

impl AsRef<Config> for StateInner {
    fn as_ref(&self) -> &Config {
        &self.config
    }
}

impl AsRef<DB> for StateInner {
    fn as_ref(&self) -> &DB {
        &self.db
    }
}

/* subject to change */
pub type State = Arc<StateInner>;
