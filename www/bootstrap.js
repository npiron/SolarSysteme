// SOLARA — WASM bootstrap
// Loads the compiled WASM module and starts the application.

import init from '../pkg/solara.js';

// ── Splash-screen helpers (called from Rust via wasm-bindgen) ────────────

window.solaraUpdateStep = function (stepId, status) {
    const el = document.getElementById('step-' + stepId);
    if (el) {
        el.className = 'step-item ' + status;
        const icon = el.querySelector('.step-icon');
        if (icon) {
            if (status === 'done')    icon.textContent = '✓';
            else if (status === 'loading') icon.textContent = '⟳';
            else                      icon.textContent = '○';
        }
    }

    // Recalculate progress
    const done  = document.querySelectorAll('.step-item.done').length;
    const total = document.querySelectorAll('.step-item').length;
    const pct   = total > 0 ? Math.round((done / total) * 100) : 0;

    const bar  = document.getElementById('progress-fill');
    const text = document.getElementById('progress-text');
    if (bar)  bar.style.width = pct + '%';
    if (text) text.textContent = pct + ' %';

    // Update status label
    const statusEl = document.getElementById('splash-status');
    if (statusEl) {
        if (pct >= 100) {
            statusEl.textContent = 'Ready!';
        } else if (status === 'loading' && el) {
            const label = el.querySelector('.step-label');
            if (label) statusEl.textContent = label.textContent + '…';
        }
    }
};

window.solaraHideSplash = function () {
    // Small delay so the user sees 100 %
    setTimeout(() => {
        const loading = document.getElementById('loading');
        if (loading) {
            loading.classList.add('hidden');
            setTimeout(() => loading.remove(), 1200);
        }
    }, 400);
};

// ── HUD helpers (called from Rust via wasm-bindgen) ──────────────────────

const MS_PER_DAY = 86400000;
let _hudFpsEma = 60;

window.solaraUpdateHud = function (days, speed, paused, fps) {
    // Exponential moving average for smooth FPS display
    _hudFpsEma = _hudFpsEma * 0.9 + fps * 0.1;

    // Convert J2000 days (Jan 1.5 2000 = 2000-01-01T12:00Z) to a calendar date
    const j2000Ms = Date.UTC(2000, 0, 1, 12, 0, 0);
    const simDate  = new Date(j2000Ms + days * MS_PER_DAY);

    const dateEl  = document.getElementById('hud-date');
    const speedEl = document.getElementById('hud-speed');
    const fpsEl   = document.getElementById('hud-fps');

    if (dateEl) {
        dateEl.textContent = simDate.toLocaleDateString('en-US', {
            year: 'numeric', month: 'short', day: 'numeric'
        });
    }
    if (speedEl) {
        if (paused) {
            speedEl.textContent = 'Paused';
        } else if (speed >= 365.25) {
            speedEl.textContent = '\u00d7' + (speed / 365.25).toFixed(1) + ' yr/s';
        } else {
            speedEl.textContent = '\u00d7' + speed.toFixed(1) + ' d/s';
        }
    }
    if (fpsEl) {
        fpsEl.textContent = Math.round(_hudFpsEma) + ' FPS';
    }
};

window.solaraToggleHud = function () {
    const hud = document.getElementById('hud');
    if (hud) hud.classList.toggle('hidden');
};

// ── Debounced resize ─────────────────────────────────────────────────────

let resizeTimer;
window.addEventListener('resize', () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
        window.dispatchEvent(new CustomEvent('solara-resize'));
    }, 150);
});

// ── Boot ──────────────────────────────────────────────────────────────────

async function run() {
    try {
        window.solaraUpdateStep('wasm', 'loading');
        await init();
        // init() calls start() → updates all engine steps synchronously.
        // Texture loads continue asynchronously; texture.rs hides the splash
        // when the last one finishes.
    } catch (error) {
        console.error('SOLARA failed to initialize:', error);
        const statusEl = document.getElementById('splash-status');
        if (statusEl) {
            statusEl.textContent = 'Error: ' + (error.message || error);
            statusEl.style.color = '#ff4444';
        }
    }
}

run();
