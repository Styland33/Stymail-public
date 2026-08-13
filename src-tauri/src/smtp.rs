use crate::models::SmtpProfile;
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, Message as LettreMessage, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

/// Build an SMTP transport from a profile.
/// Supports both SSL (implicit TLS) and STARTTLS.
fn build_transport(profile: &SmtpProfile) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let creds = Credentials::new(profile.user.clone(), profile.pass.clone());

    let transport = if profile.sec.eq_ignore_ascii_case("SSL") {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&profile.host)
            .map_err(|e| format!("Failed to create SSL relay: {}", e))?
            .credentials(creds)
            .port(profile.port)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&profile.host)
            .map_err(|e| format!("Failed to create STARTTLS relay: {}", e))?
            .credentials(creds)
            .port(profile.port)
            .build()
    };

    Ok(transport)
}

/// Test an SMTP connection and authentication.
#[tauri::command]
pub async fn test_smtp(
    host: String,
    port: u16,
    user: String,
    pass: String,
    sec: String,
) -> Result<String, String> {
    let profile = SmtpProfile {
        host,
        port,
        user,
        pass,
        name: String::new(),
        email: String::new(),
        sec,
    };

    let mailer = build_transport(&profile)?;

    match mailer.test_connection().await {
        Ok(true) => Ok("Connection and authentication successful!".to_string()),
        Ok(false) => Err("Connected, but server validation checks failed.".to_string()),
        Err(e) => Err(format!("SMTP Error: {}", e)),
    }
}

/// Send a single email using the given profile.
pub async fn send_email(
    profile: &SmtpProfile,
    to_email: &str,
    to_name: &str,
    subject: &str,
    body: &str,
    is_html: bool,
    attachment_path: Option<&str>,
) -> Result<(), String> {
    let mailer = build_transport(profile)?;

    // Build sender mailbox.
    // `email` is the From address (may differ from the auth `user`).
    // Falls back to `user` if `email` is empty (e.g. Gmail where they're the same).
    let from_address = if profile.email.is_empty() {
        profile.user.clone()
    } else {
        profile.email.clone()
    };
    let from_name = if profile.name.is_empty() {
        from_address.clone()
    } else {
        profile.name.clone()
    };
    let from = Mailbox::new(
        Some(from_name),
        from_address
            .parse()
            .map_err(|e| format!("Invalid sender email: {}", e))?,
    );
    let to = Mailbox::new(
        if to_name.is_empty() {
            None
        } else {
            Some(to_name.to_string())
        },
        to_email
            .parse()
            .map_err(|e| format!("Invalid recipient email: {}", e))?,
    );

    // Build message
    let builder = LettreMessage::builder().from(from).to(to).subject(subject);

    let body_content_type = if is_html {
        ContentType::TEXT_HTML
    } else {
        ContentType::TEXT_PLAIN
    };

    // Build email with optional attachment
    let email = if let Some(path) = attachment_path {
        let filename = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "attachment".to_string());
        let attachment = Attachment::new(filename).body(
            std::fs::read(path).map_err(|e| format!("Failed to read attachment: {}", e))?,
            ContentType::parse("application/octet-stream").unwrap(),
        );

        builder
            .multipart(
                MultiPart::mixed()
                    .singlepart(
                        SinglePart::builder()
                            .header(body_content_type)
                            .body(body.to_string()),
                    )
                    .singlepart(attachment),
            )
            .map_err(|e| format!("Failed to build email: {}", e))?
    } else {
        builder
            .header(body_content_type)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?
    };

    // Send
    mailer
        .send(email)
        .await
        .map_err(|e| format!("Send failed: {}", e))?;

    Ok(())
}
