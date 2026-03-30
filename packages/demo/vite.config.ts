/// <reference types="vitest" />
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@signalcanvas/diagram': resolve(__dirname, '../diagram/src/index.ts'),
    },
  },
  test: {
    environment: 'node',
  },
})
