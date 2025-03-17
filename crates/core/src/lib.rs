use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::http::Http;
use std::sync::Arc;

pub mod config;
use config::Settings;

pub struct Bot {
    pub http: Arc<Http>,
    pub application_id: ApplicationId,
    pub settings: Settings,
}

impl Bot {
    pub async fn new() -> Result<Self, BotError> {
        let settings = Settings::new().map_err(|e| BotError::Config(Box::new(e)))?;
        let http = Arc::new(Http::new(&settings.discord.token));
        
        let application_id = http.get_current_application_info()
            .await?
            .id;

        Ok(Self {
            http,
            application_id,
            settings,
        })
    }
}

#[derive(Debug)]
pub enum BotError {
    Serenity(SerenityError),
    Config(Box<dyn std::error::Error>),
}

impl From<SerenityError> for BotError {
    fn from(err: SerenityError) -> Self {
        BotError::Serenity(err)
    }
} 