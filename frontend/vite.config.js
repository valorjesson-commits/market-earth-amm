import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    port: 3000,
  },
  define: {
    // Some libraries might require global to be defined in the browser environment
    global: 'window',
  },
});
