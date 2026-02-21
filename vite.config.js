import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
    base: process.env.GITHUB_ACTIONS ? '/SolarSysteme/' : '/',
    plugins: [
        wasm(),
        topLevelAwait(),
    ],
    server: {
        port: 3000,
        open: false,
    },
    build: {
        outDir: '../dist',
        emptyOutDir: true,
    },
    resolve: {
        alias: {
            // Allow importing from pkg/ in the project root
        },
    },
});
