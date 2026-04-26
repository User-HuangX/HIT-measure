<template>
  <div class="dashboard">
    <div class="bg-mesh" />

    <!-- Custom title bar -->
    <header class="titlebar">
      <div class="titlebar-drag" data-tauri-drag-region>
        <span class="titlebar-icon">🔧</span>
        <span class="titlebar-app-name">HIT-<span class="titlebar-accent">measure</span></span>
        <span v-if="activeDrone" class="titlebar-drone"> / {{ activeDrone }}</span>
      </div>
      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="minimizeWindow" title="Minimize">
          <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 6h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
        </button>
        <button class="titlebar-btn" @click="toggleMaximize" title="Maximize">
          <svg width="12" height="12" viewBox="0 0 12 12"><rect x="2" y="2" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.2"/></svg>
        </button>
        <button class="titlebar-btn titlebar-btn-close" @click="closeWindow" title="Close">
          <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
        </button>
      </div>
    </header>

    <div class="app-body">
      <!-- Sidebar -->
      <DroneSidebar
        :profiles="profiles"
        :active-drone="activeDrone"
        @add="showForm = true"
        @edit="onEdit"
        @delete="onDelete"
        @switch="onSwitch"
        @settings="showSettings = true"
      />

      <!-- Main content -->
      <main class="main-grid">
        <!-- Left: Video preview -->
        <section class="glass-card video-card">
          <div class="card-header">
            <div class="card-header-left">
              <Camera class="card-icon" :size="18" />
              <span class="card-title">Live Preview</span>
            </div>
            <div class="card-badge" :class="videoRunning ? 'badge-active' : 'badge-idle'">
              {{ videoRunning ? 'ACTIVE' : 'OFFLINE' }}
            </div>
          </div>
          <div class="video-wrapper">
            <template v-if="!tauriEnv">
              <div class="video-placeholder">
                <Monitor class="placeholder-icon" :size="48" />
                <p class="placeholder-text">仅在 Tauri 窗口中可用</p>
              </div>
            </template>
            <template v-else-if="!activeDrone">
              <div class="video-placeholder">
                <Monitor class="placeholder-icon" :size="32" />
                <p class="placeholder-text">请先添加无人机</p>
              </div>
            </template>
            <template v-else-if="videoError">
              <div class="video-placeholder">
                <AlertCircle class="placeholder-icon error" :size="32" />
                <p class="placeholder-text">{{ videoError }}</p>
              </div>
            </template>
            <template v-else>
              <img
                class="video-feed"
                :src="mjpegSrc"
                alt="RTSP Preview"
                @error="onVideoError"
                @load="onVideoLoad"
              />
            </template>
          </div>
        </section>

        <!-- Right: Sensor cards -->
        <section class="sensor-stack">
          <!-- Temperature -->
          <div class="glass-card sensor-card" :class="{ 'card-flash': flashTemp }">
            <div class="card-header">
              <div class="card-header-left">
                <Thermometer class="card-icon icon-warm" :size="18" />
                <span class="card-title">Temperature</span>
              </div>
              <span class="sensor-unit">°C</span>
            </div>
            <div class="sensor-value" :style="{ '--accent': '#f59e0b' }">
              <span class="value-number">{{ formatValue(sample?.temperature, '—') }}</span>
            </div>
            <div class="progress-track">
              <div
                class="progress-fill progress-warm"
                :style="{ width: clampPercent((sample?.temperature ?? -10) / 60 * 100) }"
              />
            </div>
            <div class="sensor-range">
              <span>-10°C</span>
              <span>50°C</span>
            </div>
          </div>

          <!-- Humidity -->
          <div class="glass-card sensor-card" :class="{ 'card-flash': flashHumi }">
            <div class="card-header">
              <div class="card-header-left">
                <Droplets class="card-icon icon-blue" :size="18" />
                <span class="card-title">Humidity</span>
              </div>
              <span class="sensor-unit">%</span>
            </div>
            <div class="sensor-value" :style="{ '--accent': '#3b82f6' }">
              <span class="value-number">{{ formatValue(sample?.humidity, '—') }}</span>
            </div>
            <div class="progress-track">
              <div
                class="progress-fill progress-blue"
                :style="{ width: clampPercent(sample?.humidity) }"
              />
            </div>
            <div class="sensor-range">
              <span>0%</span>
              <span>100%</span>
            </div>
          </div>

          <!-- Photoelectric -->
          <div class="glass-card sensor-card" :class="{ 'card-flash': flashPhoto }">
            <div class="card-header">
              <div class="card-header-left">
                <Sun class="card-icon icon-amber" :size="18" />
                <span class="card-title">Photoelectric</span>
              </div>
              <span class="sensor-unit">lux</span>
            </div>
            <div class="sensor-value" :style="{ '--accent': '#f59e0b' }">
              <span class="value-number">{{ formatValue(sample?.photoelectric, '—') }}</span>
            </div>
            <div class="progress-track">
              <div
                class="progress-fill progress-amber"
                :style="{ width: clampPercent((sample?.photoelectric ?? 0) / 200 * 100) }"
              />
            </div>
            <div class="sensor-range">
              <span>0 lux</span>
              <span>200 lux</span>
            </div>
          </div>
        </section>
      </main>
    </div>

    <!-- Profile form modal -->
    <DroneProfileForm
      v-if="showForm"
      :profile="editingProfile"
      @save="onSaveProfile"
      @cancel="showForm = false"
    />
    <GlobalSettingsModal
      v-if="showSettings"
      @saved="showSettings = false"
      @cancel="showSettings = false"
    />
  </div>
</template>

<script setup>
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'
import DroneSidebar from './components/DroneSidebar.vue'
import DroneProfileForm from './components/DroneProfileForm.vue'
import GlobalSettingsModal from './components/GlobalSettingsModal.vue'
import {
  Camera,
  Thermometer,
  Droplets,
  Sun,
  AlertCircle,
  Monitor
} from 'lucide-vue-next'

const tauriEnv = ref(false)
const mjpegSrc = ref('')
const videoError = ref('')
const videoRunning = ref(false)
const sample = ref(null)
const profiles = ref([])
const activeDrone = ref(null)
const showForm = ref(false)
const showSettings = ref(false)
const editingProfile = ref(null)

let mjpegTimer = null
let unlistenMeasure = () => {}

const flashTemp = ref(false)
const flashHumi = ref(false)
const flashPhoto = ref(false)

function triggerFlash(key) {
  const map = { temperature: 'flashTemp', humidity: 'flashHumi', photoelectric: 'flashPhoto' }
  const refName = map[key]
  if (!refName) return
  const r = { flashTemp, flashHumi, flashPhoto }[refName]
  r.value = true
  setTimeout(() => { r.value = false }, 400)
}

function formatValue(val, fallback) {
  if (val === null || val === undefined) return fallback
  return Number(val).toFixed(1)
}

function clampPercent(v) {
  if (v === null || v === undefined || isNaN(v)) return '0%'
  return Math.max(0, Math.min(100, v)) + '%'
}

function onVideoError() {
  videoError.value = 'RTSP 预览加载失败'
  videoRunning.value = false
}

function onVideoLoad() {
  videoRunning.value = true
  videoError.value = ''
}

function minimizeWindow() {
  if (isTauri()) getCurrentWindow().minimize()
}

function toggleMaximize() {
  if (isTauri()) getCurrentWindow().toggleMaximize()
}

function closeWindow() {
  if (isTauri()) getCurrentWindow().close()
}

function startMjpegPoll(base) {
  clearInterval(mjpegTimer)
  const tick = () => {
    if (!base) return
    mjpegSrc.value = `${base}?t=${Date.now()}`
  }
  tick()
  mjpegTimer = setInterval(tick, 120)
}

async function switchVideoTo(droneName) {
  if (!isTauri()) return
  try {
    const { last_jpeg } = await invoke('mjpeg_asset_path_for_drone', { droneName })
    const url = convertFileSrc(last_jpeg)
    startMjpegPoll(url)
    videoError.value = ''
  } catch (e) {
    videoError.value = `视频: ${e}`
  }
}

async function onSwitch(name) {
  try {
    await invoke('switch_active_drone', { name })
    activeDrone.value = name
    await switchVideoTo(name)
  } catch (e) {
    console.error('switch drone:', e)
  }
}

async function onEdit(profile) {
  editingProfile.value = profile
  showForm.value = true
}

async function onDelete(name) {
  if (!confirm(`确定删除无人机 "${name}"？`)) return
  try {
    await invoke('delete_drone_profile', { name })
    profiles.value = await invoke('get_drone_profiles')
    const newActive = await invoke('get_active_drone')
    activeDrone.value = newActive
    sample.value = null
    if (newActive) {
      await switchVideoTo(newActive)
    } else {
      mjpegSrc.value = ''
      videoRunning.value = false
    }
  } catch (e) {
    console.error('delete drone:', e)
  }
}

async function onSaveProfile(profileData) {
  try {
    await invoke('save_drone_profile', { profile: profileData })
    profiles.value = await invoke('get_drone_profiles')
    // 如果是新增且当前没有 active drone，自动设为 active
    if (!activeDrone.value) {
      const newActive = await invoke('get_active_drone')
      activeDrone.value = newActive
      if (newActive) await switchVideoTo(newActive)
    }
    showForm.value = false
    editingProfile.value = null
  } catch (e) {
    console.error('save profile:', e)
  }
}

onMounted(async () => {
  tauriEnv.value = isTauri()

  try {
    profiles.value = await invoke('get_drone_profiles')
    activeDrone.value = await invoke('get_active_drone')
  } catch (e) {
    console.error('load profiles:', e)
  }

  if (activeDrone.value) {
    await switchVideoTo(activeDrone.value)
  }

  try {
    unlistenMeasure = await listen('measure', (ev) => {
      const { drone_name, sample: s } = ev.payload
      if (drone_name !== activeDrone.value) return
      const prev = sample.value
      if (prev) {
        if (s.temperature !== prev.temperature) triggerFlash('temperature')
        if (s.humidity !== prev.humidity) triggerFlash('humidity')
        if (s.photoelectric !== prev.photoelectric) triggerFlash('photoelectric')
      }
      sample.value = {
        temperature: s.temperature,
        humidity: s.humidity,
        photoelectric: s.photoelectric,
      }
    })
  } catch (e) {
    console.error('listen measure:', e)
  }
})

onUnmounted(() => {
  clearInterval(mjpegTimer)
  mjpegTimer = null
  unlistenMeasure()
})
</script>

<style scoped>
/* ── App body (sidebar + content) ── */
.app-body {
  position: relative;
  z-index: 1;
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}

/* ── 标题栏中的无人机名称 */
.titlebar-drone {
  font-size: 12px;
  font-weight: 400;
  color: #475569;
  margin-left: 4px;
}

/* ── 标题栏 (保留原有样式) ── */
.titlebar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 4px 0 12px;
  background: rgba(10, 10, 15, 0.95);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  user-select: none;
  -webkit-user-select: none;
}
.titlebar-drag {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  -webkit-app-region: drag;
}
.titlebar-icon {
  font-size: 16px;
  line-height: 1;
}
.titlebar-app-name {
  font-size: 13px;
  font-weight: 600;
  color: #e2e8f0;
  letter-spacing: 0.02em;
}
.titlebar-accent {
  background: linear-gradient(90deg, #f59e0b, #3b82f6, #10b981);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  -webkit-app-region: no-drag;
}
.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 30px;
  border: none;
  background: transparent;
  color: #94a3b8;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}
.titlebar-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
}
.titlebar-btn-close:hover {
  background: #ef4444;
  color: #fff;
}

/* ── Layout ── */
.dashboard {
  position: relative;
  height: 100vh;
  width: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: #0a0a0f;
}

.bg-mesh {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse 80% 50% at 20% 80%, rgba(59, 130, 246, 0.08) 0%, transparent 60%),
    radial-gradient(ellipse 60% 40% at 80% 20%, rgba(245, 158, 11, 0.06) 0%, transparent 60%),
    radial-gradient(ellipse 50% 50% at 50% 50%, rgba(139, 92, 246, 0.04) 0%, transparent 70%);
  pointer-events: none;
  z-index: 0;
}

/* ── Main grid ── */
.main-grid {
  position: relative;
  z-index: 1;
  flex: 1;
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 16px;
  padding: 16px 20px;
  min-height: 0;
  overflow: hidden;
}

/* ── Glass card ── */
.glass-card {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 16px;
  padding: 18px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

/* ── Card header ── */
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.card-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-icon {
  color: #64748b;
  flex-shrink: 0;
}

.icon-warm { color: #f59e0b; }
.icon-blue { color: #3b82f6; }
.icon-amber { color: #f59e0b; }

.card-title {
  font-size: 13px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.card-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 6px;
  letter-spacing: 0.08em;
}

.badge-active {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.badge-idle {
  background: rgba(100, 116, 139, 0.15);
  color: #64748b;
  border: 1px solid rgba(100, 116, 139, 0.2);
}

/* ── Video card ── */
.video-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.video-wrapper {
  flex: 1;
  border-radius: 12px;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.04);
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
}

.video-feed {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 12px;
}

.video-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px;
}

.placeholder-icon {
  color: #334155;
}

.placeholder-icon.error {
  color: #ef4444;
}

.placeholder-text {
  font-size: 13px;
  color: #475569;
  text-align: center;
}

/* ── Sensor stack ── */
.sensor-stack {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 0;
}

.sensor-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.sensor-unit {
  font-size: 12px;
  color: #475569;
  font-family: var(--font-mono);
  font-weight: 600;
}

.sensor-value {
  margin-bottom: 12px;
}

.value-number {
  font-size: 36px;
  font-weight: 700;
  letter-spacing: -0.03em;
  color: #e2e8f0;
  transition: color 0.3s ease;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}

/* ── Progress bar ── */
.progress-track {
  width: 100%;
  height: 4px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 6px;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  animation: shimmer 2s ease-in-out infinite;
  background-size: 200% 100%;
}

.progress-warm {
  background: linear-gradient(90deg, #f59e0b, #f97316);
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.3);
}

.progress-blue {
  background: linear-gradient(90deg, #3b82f6, #06b6d4);
  box-shadow: 0 0 12px rgba(59, 130, 246, 0.3);
}

.progress-amber {
  background: linear-gradient(90deg, #f59e0b, #eab308);
  box-shadow: 0 0 12px rgba(245, 158, 11, 0.3);
}

.sensor-range {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: #334155;
  font-family: var(--font-mono);
}

/* ── Card flash animation ── */
.card-flash::after {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--accent, rgba(59, 130, 246, 0.1));
  border-radius: 16px;
  opacity: 0;
  animation: flash-in 0.4s ease-out;
  pointer-events: none;
}

@keyframes flash-in {
  0% { opacity: 0.4; }
  100% { opacity: 0; }
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}
</style>
