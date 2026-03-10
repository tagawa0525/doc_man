use std::future::Future;
use std::pin::Pin;

/// メール送信のエラー型
#[derive(Debug)]
pub struct MailError(pub String);

impl std::fmt::Display for MailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MailError: {}", self.0)
    }
}

/// メール受信者の情報
pub struct MailRecipient {
    pub name: String,
    pub email: String,
}

/// 配布メールのコンテキスト
pub struct DistributionMailContext {
    pub doc_number: String,
    pub title: String,
    pub directory_path: String,
    pub distributed_by_name: String,
}

/// メール送信のトレイト（dyn dispatch 対応のため `Pin<Box<dyn Future>>` パターン）
pub trait MailSender: Send + Sync {
    fn send_distribution(
        &self,
        recipients: &[MailRecipient],
        context: &DistributionMailContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), MailError>> + Send + '_>>;
}

/// スタブ実装（tracing::info でログ出力のみ）
pub struct StubMailSender;

impl MailSender for StubMailSender {
    fn send_distribution(
        &self,
        recipients: &[MailRecipient],
        context: &DistributionMailContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), MailError>> + Send + '_>> {
        let recipient_names: Vec<&str> = recipients.iter().map(|r| r.name.as_str()).collect();
        tracing::info!(
            doc_number = %context.doc_number,
            title = %context.title,
            directory = %context.directory_path,
            distributed_by = %context.distributed_by_name,
            recipients = ?recipient_names,
            "stub: distribution mail would be sent"
        );
        Box::pin(async { Ok(()) })
    }
}
