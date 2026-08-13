use serde::{Deserialize, Serialize};

/// SMTP identity profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpProfile {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub name: String,
    pub email: String,
    pub sec: String, // "SSL" or "TLS"
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub workers: usize,
    pub delay_secs: u64,
    pub rounds: u32,
    pub round_delay_secs: u64,
    pub max_retries: u32,
    pub retry_delay_secs: u64,
    pub random_delay: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            workers: 1,
            delay_secs: 5,
            rounds: 1,
            round_delay_secs: 60,
            max_retries: 0,
            retry_delay_secs: 10,
            random_delay: true,
        }
    }
}

/// A single recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub email: String,
    pub name: String,
}

/// Campaign message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub subject: String,
    pub body: String,
    pub is_html: bool,
    pub attachment: Option<String>,
}

/// Full campaign payload sent from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignPayload {
    pub mode: String, // "single" or "pool"
    pub smtp_pool: Vec<SmtpProfile>,
    pub config: EngineConfig,
    pub message: Message,
    pub recipients: Vec<Recipient>,
}

/// Live campaign stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignStats {
    pub sent: u64,
    pub failed: u64,
    pub total: u64,
    pub current_round: u32,
    pub total_rounds: u32,
    pub progress: f64,
    pub running: bool,
    pub paused: bool,
}

impl Default for CampaignStats {
    fn default() -> Self {
        Self {
            sent: 0,
            failed: 0,
            total: 0,
            current_round: 0,
            total_rounds: 0,
            progress: 0.0,
            running: false,
            paused: false,
        }
    }
}

/// Log entry emitted to the frontend console
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String, // INFO, SYSTEM, SUCCESS, WARNING, DANGER, ERROR, RETRY
    pub message: String,
}

/// Project file structure for save/load
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub version: String,
    pub saved_at: String,
    pub smtp_pool: Vec<SmtpProfile>,
    pub config: EngineConfig,
    pub message: Message,
    pub recipients: Vec<Recipient>,
}
