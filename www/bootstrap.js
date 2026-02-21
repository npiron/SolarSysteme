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
