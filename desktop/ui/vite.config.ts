import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    fs: {
      // The dev harness serves the branding logo from desktop/resources.
      allow: ['.', '../resources'],
    },
  },
  build: {
    // Keep the output scannable by the no-external-requests check.
    sourcemap: false,
  },
  test: {
    environment: 'jsdom',
    setupFiles: './src/test-setup.ts',
  },
})
