import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const roomServerPort = Number.parseInt(process.env.ROOM_SERVER_PORT, 10) || 5050;

export default defineConfig({
  plugins: [svelte()],
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/target/**'] },
    proxy: {
      '/api/audio': {
        target: 'http://127.0.0.1:8787',
        rewrite: (path) => path.replace(/^\/api\/audio/, ''),
      },
      '/api': `http://127.0.0.1:${roomServerPort}`,
      '/socket.io': { target: `http://127.0.0.1:${roomServerPort}`, ws: true },
    }
  },
  build: { outDir: 'dist' }
});
