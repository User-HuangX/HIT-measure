//vite配置文件 —— 端口与 SSE 地址均来自根目录 `.env`，勿在此写死
import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig(({ mode }) => {
  const root = process.cwd()
  const env = loadEnv(mode, root, '')
  const port = Number(env.DEV_SERVER_PORT || 5180)
  const ssePort = env.SSE_PORT || '5888'
  const sseUrl = `http://127.0.0.1:${ssePort}/events`

  return {
    plugins: [vue()],
    server: {
      port,
      strictPort: true,
    },
    define: {
      'import.meta.env.VITE_SSE_URL': JSON.stringify(sseUrl),
    },
  }
})
