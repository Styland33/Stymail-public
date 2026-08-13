// ═══════════════════════════════════════════════════════════
// STYMAIL PRO — Frontend Logic
// ═══════════════════════════════════════════════════════════

// ── State ───────────────────────────────────────────────────
const state = {
    smtpPool: [],
    editingIndex: null,
    isRunning: false,
    isPaused: false
};

// ── Element Helpers ─────────────────────────────────────────
const $ = (id) => document.getElementById(id);
const $q = (sel) => document.querySelector(sel);
const $qa = (sel) => document.querySelectorAll(sel);

// ── Toast System ────────────────────────────────────────────
const TOAST_ICONS = {
    success: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>',
    error: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>',
    info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>',
    warning: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>'
};

function showToast(message, type = 'info', duration = 3000) {
    const container = $('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `${TOAST_ICONS[type] || TOAST_ICONS.info}<span>${message}</span>`;
    container.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('leaving');
        setTimeout(() => toast.remove(), 250);
    }, duration);
}

// ── Tab Navigation ──────────────────────────────────────────
function switchTab(targetId) {
    // Update nav items (both sidebar and mobile)
    $qa('.nav-item').forEach(item => {
        item.classList.toggle('active', item.dataset.target === targetId);
    });

    // Update panels
    $qa('.tab-panel').forEach(panel => {
        panel.classList.toggle('active', panel.id === targetId);
    });

    // Update topbar title/subtitle
    const activeNav = $q(`.nav-item[data-target="${targetId}"]`);
    if (activeNav) {
        $('page-title').textContent = activeNav.dataset.title || '';
        $('page-subtitle').textContent = activeNav.dataset.subtitle || '';
    }
}

$qa('.nav-item').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.target));
});

// ── Network Indicator ───────────────────────────────────────
window.addEventListener('offline', () => {
    $('offline-indicator').classList.remove('hidden');
    showToast('Network connection lost', 'warning');
});
window.addEventListener('online', () => {
    $('offline-indicator').classList.add('hidden');
    showToast('Network connection restored', 'success');
});

// ── SMTP Modal ──────────────────────────────────────────────
function openSmtpModal() {
    $('smtp-modal').classList.remove('hidden');
    document.body.style.overflow = 'hidden';
}

function closeSmtpModal() {
    $('smtp-modal').classList.add('hidden');
    document.body.style.overflow = '';
    cancelSmtpEdit();
}

$('btn-manage-pool').addEventListener('click', openSmtpModal);
$('btn-close-modal').addEventListener('click', closeSmtpModal);

// Close via backdrop or X button
$qa('[data-close-modal]').forEach(el => {
    el.addEventListener('click', closeSmtpModal);
});

// Close on Escape key
document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && !$('smtp-modal').classList.contains('hidden')) {
        closeSmtpModal();
    }
});

$('btn-smtp-save').addEventListener('click', () => {
    const smtpData = {
        host: $('smtp-host').value.trim(),
        port: parseInt($('smtp-port').value.trim(), 10) || 465,
        user: $('smtp-user').value.trim(),
        pass: $('smtp-pass').value,
        name: $('smtp-name').value.trim(),
        email: $('smtp-email').value.trim(),
        sec: $('smtp-sec').value
    };

    if (!smtpData.host || !smtpData.user) {
        showToast('Host and User are required', 'error');
        return;
    }

    if (state.editingIndex !== null) {
        state.smtpPool[state.editingIndex] = smtpData;
        showToast('SMTP profile updated', 'success');
        cancelSmtpEdit();
    } else {
        state.smtpPool.push(smtpData);
        showToast('SMTP profile added to pool', 'success');
    }

    clearSmtpForm();
    renderSmtpList();
});

$('btn-smtp-cancel-edit').addEventListener('click', cancelSmtpEdit);

function cancelSmtpEdit() {
    state.editingIndex = null;
    $('smtp-modal-title').innerText = 'Add SMTP Identity';
    $('btn-smtp-save').querySelector('.btn-label').innerText = 'Add to Pool';
    $('btn-smtp-cancel-edit').classList.add('hidden');
    clearSmtpForm();
}

function clearSmtpForm() {
    $('smtp-host').value = '';
    $('smtp-port').value = '';
    $('smtp-user').value = '';
    $('smtp-pass').value = '';
    $('smtp-name').value = '';
    $('smtp-email').value = '';
    $('smtp-sec').value = 'SSL';
}

function renderSmtpList() {
    const container = $('smtp-list');
    container.innerHTML = '';

    // Update count badge
    $('smtp-count').textContent = state.smtpPool.length;

    if (state.smtpPool.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                <p>No SMTP identities yet</p>
                <small>Add your first profile above</small>
            </div>
        `;
        return;
    }

    state.smtpPool.forEach((smtp, idx) => {
        const item = document.createElement('div');
        item.className = 'smtp-item';

        const emailDisplay = smtp.email ? ` · From: ${smtp.email}` : '';

        const lt = '\u003C';
        const gt = '\u003E';
        item.innerHTML = `
            <div class="smtp-item-info">
                <div class="smtp-item-name">${escapeHtml(smtp.name || smtp.user)} ${lt}${escapeHtml(smtp.user)}${gt}</div>
                <div class="smtp-item-meta">${escapeHtml(smtp.host)}:${escapeHtml(smtp.port)} (${escapeHtml(smtp.sec)})${emailDisplay}</div>
            </div>
            <div class="smtp-item-actions">
                <button class="btn-edit" onclick="editSmtp(${idx})" title="Edit">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
                </button>
                <button class="btn-del" onclick="deleteSmtp(${idx})" title="Delete">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
            </div>
        `;
        container.appendChild(item);
    });
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

window.editSmtp = function(idx) {
    state.editingIndex = idx;
    const smtp = state.smtpPool[idx];

    $('smtp-host').value = smtp.host;
    $('smtp-port').value = smtp.port;
    $('smtp-user').value = smtp.user;
    $('smtp-pass').value = smtp.pass;
    $('smtp-name').value = smtp.name;
    $('smtp-email').value = smtp.email;
    $('smtp-sec').value = smtp.sec;

    $('smtp-modal-title').innerText = `Editing: ${smtp.user}`;
    $('btn-smtp-save').querySelector('.btn-label').innerText = 'Update Profile';
    $('btn-smtp-cancel-edit').classList.remove('hidden');
};

window.deleteSmtp = function(idx) {
    if (state.editingIndex === idx) cancelSmtpEdit();
    if (state.editingIndex !== null && idx < state.editingIndex) state.editingIndex--;

    state.smtpPool.splice(idx, 1);
    renderSmtpList();
    showToast('SMTP profile removed', 'info');
};

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ── SMTP Test ────────────────────────────────────────────────
$('btn-smtp-test').addEventListener('click', async () => {
    const host = $('smtp-host').value.trim();
    const port = parseInt($('smtp-port').value.trim(), 10);
    const user = $('smtp-user').value.trim();
    const pass = $('smtp-pass').value;
    const sec = $('smtp-sec').value;

    if (!host || !user) {
        showToast('Host and User are required to test', 'error');
        return;
    }

    logToConsole(`Testing SMTP connection to ${host}:${port}...`, 'SYSTEM');
    showToast('Testing connection…', 'info');

    try {
        const result = await invoke('test_smtp', { host, port, user, pass, sec });
        logToConsole(result, 'SUCCESS');
        showToast(result, 'success');
    } catch (e) {
        logToConsole(`SMTP test failed: ${e}`, 'DANGER');
        showToast(`Connection failed: ${e}`, 'error');
    }
});

$('btn-attach').addEventListener('click', async () => {
    try {
        const selected = await invoke('plugin:dialog|open', {
            options: { multiple: false, directory: false }
        });
        if (selected) {
            const path = Array.isArray(selected) ? selected[0] : selected;
            const name = path.split(/[\\/]/).pop();
            const chip = $('attachment-name');
            chip.textContent = `📎 ${name}`;
            chip.classList.remove('hidden');
            chip.dataset.path = path;
            showToast('Attachment added', 'success');
        }
    } catch (e) {
        showToast(`Failed to pick attachment: ${e}`, 'error');
    }
});

function gatherProjectData() {
    return {
        smtp_pool: state.smtpPool,
        config: {
            workers: parseInt($('cfg-workers').value, 10) || 1,
            delay_secs: parseInt($('cfg-delay').value, 10) || 5,
            rounds: parseInt($('cfg-rounds').value, 10) || 1,
            round_delay_secs: parseInt($('cfg-round-delay').value, 10) || 60,
            max_retries: parseInt($('cfg-retries').value, 10) || 0,
            retry_delay_secs: parseInt($('cfg-retry-delay').value, 10) || 10,
            random_delay: $('cfg-random-delay').checked
        },
        message: {
            subject: $('cmp-subject').value,
            body: $('cmp-body').value,
            is_html: $('cmp-is-html').checked,
            attachment: $('attachment-name').dataset.path || null
        },
        recipients: parseRecipients($('cmp-recipients').value)
    };
}

function parseRecipients(text) {
    return text.split('\n')
        .map(line => line.trim())
        .filter(Boolean)
        .map(line => {
            const parts = line.split(',').map(s => s.trim());
            return {
                email: parts[0],
                name: parts[1] || ''
            };
        });
}

function applyProjectData(project) {
    // SMTP pool
    state.smtpPool = project.smtp_pool || [];
    renderSmtpList();

    // Engine config
    const cfg = project.config || {};
    $('cfg-workers').value = cfg.workers ?? 1;
    $('cfg-delay').value = cfg.delay_secs ?? 5;
    $('cfg-rounds').value = cfg.rounds ?? 1;
    $('cfg-round-delay').value = cfg.round_delay_secs ?? 60;
    $('cfg-retries').value = cfg.max_retries ?? 0;
    $('cfg-retry-delay').value = cfg.retry_delay_secs ?? 10;
    $('cfg-random-delay').checked = cfg.random_delay ?? true;

    // Message
    const msg = project.message || {};
    $('cmp-subject').value = msg.subject || '';
    $('cmp-body').value = msg.body || '';
    $('cmp-is-html').checked = msg.is_html ?? true;
    if (msg.attachment) {
        const chip = $('attachment-name');
        chip.textContent = `📎 ${msg.attachment.split(/[\\/]/).pop()}`;
        chip.dataset.path = msg.attachment;
        chip.classList.remove('hidden');
    }

    // Recipients
    const recips = project.recipients || [];
    $('cmp-recipients').value = recips.map(r => r.name ? `${r.email}, ${r.name}` : r.email).join('\n');
}

$('btn-save-proj').addEventListener('click', async () => {
    const data = gatherProjectData();
    const project = {
        version: '0.1.0',
        saved_at: new Date().toISOString(),
        ...data
    };

    try {
        const path = await invoke('save_project', { project });
        logToConsole(`Project saved to ${path}`, 'SUCCESS');
        showToast('Project saved', 'success');
    } catch (e) {
        if (e !== 'Save cancelled.') {
            logToConsole(`Save failed: ${e}`, 'DANGER');
            showToast(`Save failed: ${e}`, 'error');
        }
    }
});

$('btn-load-proj').addEventListener('click', async () => {
    try {
        const project = await invoke('load_project');
        applyProjectData(project);
        logToConsole('Project loaded successfully', 'SUCCESS');
        showToast('Project loaded', 'success');
    } catch (e) {
        if (e !== 'Load cancelled.') {
            logToConsole(`Load failed: ${e}`, 'DANGER');
            showToast(`Load failed: ${e}`, 'error');
        }
    }
});

// ── Engine Controls ─────────────────────────────────────────
$('btn-start').addEventListener('click', async () => {
    if (state.isRunning) return;

    const mode = $q('input[name="smtp_mode"]:checked').value;
    if (mode === 'pool' && state.smtpPool.length === 0) {
        showToast('Pool is empty! Add SMTPs or switch mode.', 'error');
        return;
    }
    if (mode === 'single' && state.smtpPool.length === 0) {
        showToast('Add at least one SMTP profile first.', 'error');
        return;
    }

    const data = gatherProjectData();
    const payload = {
        mode,
        smtp_pool: state.smtpPool,
        config: data.config,
        message: data.message,
        recipients: data.recipients
    };

    if (payload.recipients.length === 0) {
        showToast('No recipients provided.', 'error');
        return;
    }

    try {
        await invoke('start_campaign', { payload });
        state.isRunning = true;
        state.isPaused = false;

        // Auto-switch to stats tab
        switchTab('tab-stats');

        // Update engine status
        $q('.status-text').textContent = 'Engine Running';
        $q('.status-pulse').style.background = 'var(--accent)';

        logToConsole('🚀 Campaign Started', 'SYSTEM');
        showToast('Campaign started', 'success');
    } catch (e) {
        logToConsole(`Failed to start: ${e}`, 'DANGER');
        showToast(`Failed to start: ${e}`, 'error');
    }
});

$('btn-pause').addEventListener('click', async () => {
    if (!state.isRunning) return;

    try {
        const paused = await invoke('toggle_pause');
        state.isPaused = paused;
        const btn = $('btn-pause');
        const label = btn.querySelector('.btn-label');

        if (paused) {
            label.textContent = 'Resume';
            btn.className = 'btn btn-success btn-lg';
            logToConsole('⏸ Campaign PAUSED.', 'WARNING');
            showToast('Campaign paused', 'warning');
        } else {
            label.textContent = 'Pause';
            btn.className = 'btn btn-warning btn-lg';
            logToConsole('▶ Campaign RESUMED.', 'SYSTEM');
            showToast('Campaign resumed', 'success');
        }
    } catch (e) {
        showToast(e, 'error');
    }
});

$('btn-kill').addEventListener('click', async () => {
    if (!state.isRunning) return;
    logToConsole('🛑 STOP SIGNAL RECEIVED. Waiting for Rust threads to safely drop...', 'DANGER');
    showToast('Stop signal sent', 'error');

    try {
        await invoke('stop_campaign');
    } catch (e) {
        // Already stopped or not running
    }

    state.isRunning = false;
    state.isPaused = false;

    // Reset engine status
    $q('.status-text').textContent = 'Engine Idle';
    $q('.status-pulse').style.background = 'var(--success)';
});

// ── Console ─────────────────────────────────────────────────
$('btn-clear-log').addEventListener('click', () => {
    $('console-log').innerHTML = '';
    logToConsole('Console cleared', 'INFO');
});

function logToConsole(message, type = 'INFO') {
    const time = new Date().toLocaleTimeString('en-US', { hour12: false });
    const logEl = $('console-log');

    let className = 'log-info';
    if (type === 'SYSTEM') className = 'log-system';
    if (type === 'SUCCESS') className = 'log-success';
    if (type === 'WARNING' || type === 'RETRY') className = 'log-warning';
    if (type === 'DANGER' || type === 'ERROR') className = 'log-danger';

    const line = document.createElement('div');
    line.className = `log-line ${className}`;
    line.textContent = `[${time}] [${type}] ${message}`;
    logEl.appendChild(line);
    logEl.scrollTop = logEl.scrollHeight;
}

// ── Progress Helper ─────────────────────────────────────────
function updateProgress(value) {
    const progress = $('campaign-progress');
    progress.value = value;
    $('progress-label').textContent = `${Math.round(value)}%`;
}

// ── Live Event Listeners ────────────────────────────────────
async function setupEventListeners() {
    try {
        await listen('campaign-log', (event) => {
            const { level, message } = event.payload;
            logToConsole(message, level);
        });

        await listen('campaign-stats', (event) => {
            const stats = event.payload;
            $('stat-sent').textContent = stats.sent;
            $('stat-fail').textContent = stats.failed;
            $('stat-round').textContent = `${stats.current_round} / ${stats.total_rounds}`;
            updateProgress(stats.progress);

            if (!stats.running && state.isRunning) {
                state.isRunning = false;
                state.isPaused = false;
                $q('.status-text').textContent = 'Engine Idle';
                $q('.status-pulse').style.background = 'var(--success)';
                $('btn-pause').querySelector('.btn-label').textContent = 'Pause';
                $('btn-pause').className = 'btn btn-warning btn-lg';
            }
        });
    } catch (e) {
        console.error('Failed to setup event listeners:', e);
    }
}

// ── Init ────────────────────────────────────────────────────
renderSmtpList();
setupEventListeners();
logToConsole('Stymail Pro ready', 'SYSTEM');
