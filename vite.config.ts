import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed port and fails if it is not available.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],

  // Vite options tailored for Tauri development.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: {
      // The Rust side is watched by the Tauri CLI, not by Vite.
      ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**'],
    },
  },

  // Produce readable output in debug builds so a stack trace from a user is useful.
  // 'oxc' rather than 'esbuild': Vite 8 deprecated the esbuild minifier and no longer
  // bundles it, so naming it here fails the build with a missing-package error.
  build: {
    target: 'esnext',
    minify: process.env.TAURI_ENV_DEBUG ? false : 'oxc',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
