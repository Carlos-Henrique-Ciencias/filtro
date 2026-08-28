import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 9000,
    strictPort: true,
    watch: {
      ignored: ['**/aprovados.json', '**/fila_trabalho.json', '**/*.html']
    }
  }
});