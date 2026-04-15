import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig(({ mode }) => {
  const root = process.cwd()
  const env = loadEnv(mode, root, '')
  const port = Number(env.DEV_SERVER_PORT || 5180)

  return {
    plugins: [vue()],
    server: {
      port,
      strictPort: true,
    },
  }
})
