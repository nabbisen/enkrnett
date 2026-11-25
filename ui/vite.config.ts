import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// The daemon serves `dist/`: `index.html` is `no-store`, every other file is content-hashed and
// immutable (ENK-API-024). Vite's default `assets/name-[hash].ext` is what the daemon recognises.
export default defineConfig({
  plugins: [svelte()],
  base: '/',
  build: {
    target: 'es2020',
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    modulePreload: { polyfill: false },
    // Hex hashes: the daemon recognises `name-<hex>.ext` as content-hashed and immutable.
    rollupOptions: { output: { manualChunks: undefined, hashCharacters: 'hex' } },
  },
  server: {
    // During development the API is the mock daemon.
    proxy: { '/api': 'http://127.0.0.1:8080' },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts', 'scripts/**/*.test.mjs'],
  },
});
