use crate::models::{CampaignPayload, CampaignStats, LogEntry};
use crate::smtp::send_email;
use crate::spintax::expand_spintax;
use chrono::Local;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Shared engine state accessible from Tauri commands.
#[derive(Clone)]
pub struct EngineState {
    inner: Arc<EngineInner>,
}

struct EngineInner {
    running: AtomicBool,
    paused: AtomicBool,
    sent: AtomicU64,
    failed: AtomicU64,
    current_round: AtomicU32,
    total_rounds: AtomicU32,
    total_emails: AtomicU64,
    log: Mutex<Vec<LogEntry>>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(EngineInner {
                running: AtomicBool::new(false),
                paused: AtomicBool::new(false),
                sent: AtomicU64::new(0),
                failed: AtomicU64::new(0),
                current_round: AtomicU32::new(0),
                total_rounds: AtomicU32::new(0),
                total_emails: AtomicU64::new(0),
                log: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn is_running(&self) -> bool {
        self.inner.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.inner.paused.load(Ordering::SeqCst)
    }

    pub fn set_running(&self, val: bool) {
        self.inner.running.store(val, Ordering::SeqCst);
    }

    pub fn set_paused(&self, val: bool) {
        self.inner.paused.store(val, Ordering::SeqCst);
    }

    pub fn increment_sent(&self) {
        self.inner.sent.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_failed(&self) {
        self.inner.failed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn set_round(&self, round: u32) {
        self.inner.current_round.store(round, Ordering::SeqCst);
    }

    pub fn set_total_rounds(&self, rounds: u32) {
        self.inner.total_rounds.store(rounds, Ordering::SeqCst);
    }

    pub fn set_total_emails(&self, total: u64) {
        self.inner.total_emails.store(total, Ordering::SeqCst);
    }

    pub fn reset_counters(&self) {
        self.inner.sent.store(0, Ordering::SeqCst);
        self.inner.failed.store(0, Ordering::SeqCst);
        self.inner.current_round.store(0, Ordering::SeqCst);
        self.inner.total_rounds.store(0, Ordering::SeqCst);
        self.inner.total_emails.store(0, Ordering::SeqCst);
        if let Ok(mut log) = self.inner.log.lock() {
            log.clear();
        }
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.inner.log.lock().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn get_stats(&self) -> CampaignStats {
        let sent = self.inner.sent.load(Ordering::SeqCst);
        let failed = self.inner.failed.load(Ordering::SeqCst);
        let total = self.inner.total_emails.load(Ordering::SeqCst);
        let current_round = self.inner.current_round.load(Ordering::SeqCst);
        let total_rounds = self.inner.total_rounds.load(Ordering::SeqCst);

        let progress = if total > 0 {
            ((sent + failed) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CampaignStats {
            sent,
            failed,
            total,
            current_round,
            total_rounds,
            progress,
            running: self.is_running(),
            paused: self.is_paused(),
        }
    }
}

/// Emit a log event to the frontend.
fn emit_log(app: &AppHandle, level: &str, message: &str) {
    let entry = LogEntry {
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        level: level.to_string(),
        message: message.to_string(),
    };
    let _ = app.emit("campaign-log", entry);
}

/// Emit updated stats to the frontend.
fn emit_stats(app: &AppHandle, state: &EngineState) {
    let _ = app.emit("campaign-stats", state.get_stats());
}

/// Start a campaign. Spawns a background task that processes all recipients.
#[tauri::command]
pub async fn start_campaign(
    app: AppHandle,
    state: tauri::State<'_, EngineState>,
    payload: CampaignPayload,
) -> Result<(), String> {
    if state.is_running() {
        return Err("Campaign is already running.".to_string());
    }

    // Validate payload
    if payload.recipients.is_empty() {
        return Err("No recipients provided.".to_string());
    }
    if payload.smtp_pool.is_empty() {
        return Err("No SMTP profiles provided.".to_string());
    }

    // Reset counters and set state
    state.reset_counters();
    state.set_running(true);
    state.set_paused(false);
    state.set_total_rounds(payload.config.rounds);
    state.set_total_emails(payload.recipients.len() as u64 * payload.config.rounds as u64);

    emit_log(&app, "SYSTEM", "🚀 Campaign started");
    emit_stats(&app, &state);

    let state_clone = (*state).clone();
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        run_campaign(app_clone, state_clone, payload).await;
    });

    Ok(())
}

/// Pause or resume the running campaign.
#[tauri::command]
pub fn toggle_pause(app: AppHandle, state: tauri::State<'_, EngineState>) -> Result<bool, String> {
    if !state.is_running() {
        return Err("Campaign is not running.".to_string());
    }

    let new_paused = !state.is_paused();
    state.set_paused(new_paused);

    if new_paused {
        emit_log(&app, "WARNING", "⏸ Campaign PAUSED.");
    } else {
        emit_log(&app, "SYSTEM", "▶ Campaign RESUMED.");
    }
    emit_stats(&app, &state);

    Ok(new_paused)
}

/// Stop the running campaign.
#[tauri::command]
pub fn stop_campaign(app: AppHandle, state: tauri::State<'_, EngineState>) -> Result<(), String> {
    if !state.is_running() {
        return Err("Campaign is not running.".to_string());
    }

    state.set_running(false);
    state.set_paused(false);
    emit_log(&app, "DANGER", "🛑 STOP SIGNAL RECEIVED. Campaign stopped.");
    emit_stats(&app, &state);

    Ok(())
}

/// Get current campaign stats.
#[tauri::command]
pub fn get_stats(state: tauri::State<'_, EngineState>) -> CampaignStats {
    state.get_stats()
}

/// Get the campaign log.
#[tauri::command]
pub fn get_logs(state: tauri::State<'_, EngineState>) -> Vec<LogEntry> {
    state.get_logs()
}

/// Core campaign loop.
async fn run_campaign(app: AppHandle, state: EngineState, payload: CampaignPayload) {
    let config = payload.config.clone();
    let recipients = payload.recipients.clone();
    let message = payload.message.clone();
    let smtp_pool = payload.smtp_pool.clone();
    let mode = payload.mode.clone();

    // Build a pool of profiles to rotate through
    let profiles: Vec<_> = if mode == "pool" {
        smtp_pool.clone()
    } else {
        // Single mode: use the first profile
        vec![smtp_pool[0].clone()]
    };

    let mut rng = StdRng::from_entropy();
    let mut profile_idx = 0usize;

    for round in 1..=config.rounds {
        if !state.is_running() {
            break;
        }

        state.set_round(round);
        emit_log(
            &app,
            "SYSTEM",
            &format!("📬 Round {}/{} started", round, config.rounds),
        );
        emit_stats(&app, &state);

        for recipient in &recipients {
            // Check if stopped
            if !state.is_running() {
                break;
            }

            // Wait while paused
            while state.is_paused() && state.is_running() {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            if !state.is_running() {
                break;
            }

            // Expand spintax for subject and body
            let subject = expand_spintax(&message.subject);
            let body = expand_spintax(&message.body);

            // Send with retries — rotate to a different SMTP profile on each retry
            let mut attempt = 0;

            loop {
                if !state.is_running() {
                    break;
                }

                // Pick profile (round-robin) — advances on every attempt so
                // retries use a different identity from the pool
                let profile = &profiles[profile_idx % profiles.len()];
                profile_idx += 1;

                match send_email(
                    profile,
                    &recipient.email,
                    &recipient.name,
                    &subject,
                    &body,
                    message.is_html,
                    message.attachment.as_deref(),
                )
                .await
                {
                    Ok(()) => {
                        state.increment_sent();
                        emit_log(
                            &app,
                            "SUCCESS",
                            &format!(
                                "✅ Sent to {} <{}> via {}",
                                recipient.name, recipient.email, profile.user
                            ),
                        );
                        break;
                    }
                    Err(e) => {
                        attempt += 1;
                        if attempt <= config.max_retries {
                            emit_log(
                                &app,
                                "RETRY",
                                &format!(
                                    "⚠️ Retry {}/{} for {} via {}: {}",
                                    attempt, config.max_retries, recipient.email, profile.user, e
                                ),
                            );
                            tokio::time::sleep(std::time::Duration::from_secs(
                                config.retry_delay_secs,
                            ))
                            .await;
                        } else {
                            state.increment_failed();
                            emit_log(
                                &app,
                                "DANGER",
                                &format!("❌ Failed {}: {}", recipient.email, e),
                            );
                            break;
                        }
                    }
                }
            }

            emit_stats(&app, &state);

            // Delay between sends
            if state.is_running() {
                let delay = if config.random_delay {
                    // Randomize between 50% and 150% of configured delay
                    let base = config.delay_secs as u64;
                    let min = base.saturating_sub(base / 2).max(1);
                    let max = base + base / 2;
                    rng.gen_range(min..=max)
                } else {
                    config.delay_secs
                };
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
        }

        // Delay between rounds (except after the last round)
        if round < config.rounds && state.is_running() {
            emit_log(
                &app,
                "SYSTEM",
                &format!(
                    "⏳ Waiting {}s before next round...",
                    config.round_delay_secs
                ),
            );
            tokio::time::sleep(std::time::Duration::from_secs(config.round_delay_secs)).await;
        }
    }

    // Campaign finished
    state.set_running(false);
    state.set_paused(false);
    emit_log(&app, "SYSTEM", "🏁 Campaign completed.");
    emit_stats(&app, &state);
}
