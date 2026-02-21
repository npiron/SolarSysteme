// SOLARA — WASM bootstrap
// Loads the compiled WASM module and starts the application.

import init from '../pkg/solara.js';

async function run() {
    try {
        await init();
        // Hide loading screen once WASM is initialized and running
        const loading = document.getElementById('loading');
        if (loading) {
            loading.classList.add('hidden');
            setTimeout(() => loading.remove(), 1000);
        }
    } catch (error) {
        console.error('SOLARA failed to initialize:', error);
        const loading = document.getElementById('loading');
        if (loading) {
            const sub = loading.querySelector('.loader-sub');
            if (sub) {
                sub.textContent = `Error: ${error.message || error}`;
                sub.style.color = '#ff4444';
            }
        }
    }
}

run();
