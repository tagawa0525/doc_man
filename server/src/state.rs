use std::sync::Arc;

use sqlx::PgPool;

use crate::services::mail::MailSender;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub mail_sender: Arc<dyn MailSender>,
}
