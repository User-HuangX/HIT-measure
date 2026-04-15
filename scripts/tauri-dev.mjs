#!/usr/bin/env node
/** 读根目录 `.env` 的 `DEV_SERVER_PORT`，合并 Tauri `build.devUrl`，与 Vite 一致。 */
import { spawn } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), '..')
const envPath = join(projectRoot, '.env')

function parseEnvFile(path) {
  if (!existsSync(path)) return {}
  const out = {}
  for (const line of readFileSync(path, 'utf8').split(/\r?\n/)) {
    const t = line.trim()
    if (!t || t.startsWith('#')) continue
    const eq = t.indexOf('=')
    if (eq === -1) continue
    let v = t.slice(eq + 1).trim()
    if (
      (v.startsWith('"') && v.endsWith('"')) ||
      (v.startsWith("'") && v.endsWith("'"))
    ) {
      v = v.slice(1, -1)
    }
    out[t.slice(0, eq).trim()] = v
  }
  return out
}

const e = parseEnvFile(envPath)
const port = e.DEV_SERVER_PORT || '5180'
const merge = JSON.stringify({ build: { devUrl: `http://localhost:${port}` } })
const args = process.argv.slice(2)
const child = spawn('cargo', ['tauri', ...args, '--config', merge], {
  cwd: projectRoot,
  stdio: 'inherit',
  shell: false,
})
child.on('exit', (code) => process.exit(code ?? 1))
