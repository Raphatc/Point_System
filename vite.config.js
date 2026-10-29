import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

const buildStamp = new Date().toISOString().slice(0, 10).replaceAll('-', '.')

export default defineConfig({
  define: {
    __APP_BUILD__: JSON.stringify(buildStamp),
  },
  plugins: [vue()],
  server: {
    host: '0.0.0.0',
    port: 1420,
  },
  preview: {
    host: '0.0.0.0',
    port: 4173,
  },
})
