<template>
    <div>
        <h1>HIT-measure</h1>

        <section class="card">
            <h2>无人机视角</h2>
            <p v-if="!tauriEnv" class="err">
                请在 Tauri 窗口中运行（HLS / MJPEG 使用 Asset Protocol 读本地文件）。
            </p>
            <template v-else>
                <video
                    v-show="streamMode === 'hls'"
                    ref="hlsVideoEl"
                    class="hls-video"
                    controls
                    playsinline
                    muted
                />
                <img
                    v-show="streamMode === 'mjpeg'"
                    class="hls-video mjpeg-preview"
                    :src="mjpegSrc"
                    alt="MJPEG 预览"
                    @error="onMjpegImgError"
                />
                <p v-if="hlsError" class="err">{{ hlsError }}</p>
            </template>
        </section>

        <section class="card">
            <h2>测量（MQTT → SSE）</h2>
            <p v-if="sseError">{{ sseError }}</p>
            <div v-else-if="parseError" class="err">{{ parseError }}</div>
            <div v-else-if="sample">
                <p>温度：{{ sample.temperature }}</p>
                <p>湿度：{{ sample.humidity }}</p>
                <p>光电：{{ sample.photoelectric }}</p>
                <pre class="raw">{{ rawPayload }}</pre>
            </div>
            <p v-else>等待 SSE（{{ sseUrl }}）…</p>
        </section>
    </div>
</template>

<script setup>
import Hls from 'hls.js'
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'

const streamPort = import.meta.env.VITE_STREAM_PORT || '5888'

function streamOrigin() {
    const override = import.meta.env.VITE_STREAM_API_ORIGIN
    if (override) {
        return String(override).replace(/\/$/, '')
    }
    if (typeof window === 'undefined') {
        return `http://127.0.0.1:${streamPort}`
    }
    return `${window.location.protocol}//${window.location.hostname}:${streamPort}`
}

const sseUrl = `${streamOrigin()}/events`

const tauriEnv = ref(false)
const hlsUrl = ref('')
const mjpegSnapUrl = ref('')

function initialStreamMode() {
    if (Hls.isSupported()) return 'hls'
    const v = document.createElement('video')
    if (v.canPlayType('application/vnd.apple.mpegurl')) return 'hls'
    return 'mjpeg'
}

/** @type {import('vue').Ref<'hls' | 'mjpeg'>} */
const streamMode = ref(initialStreamMode())

const hlsVideoEl = ref(null)
const hlsError = ref('')
const mjpegSrc = ref('')
let hlsPlayer = null
let mjpegTimer = null

const sample = ref(null)
const rawPayload = ref('')
const parseError = ref('')
const sseError = ref('')
let es = null

function onMjpegImgError() {
    if (streamMode.value !== 'mjpeg') return
    hlsError.value = `MJPEG 无法加载（${mjpegSnapUrl.value}）。若刚启动请稍等；并确认 RTSP_RELAY_ENABLED、ffmpeg 与 RTSP。`
}

function startMjpegPoll() {
    clearInterval(mjpegTimer)
    hlsError.value = ''
    const tick = () => {
        const base = mjpegSnapUrl.value
        if (!base) return
        mjpegSrc.value = `${base}?t=${Date.now()}`
    }
    tick()
    mjpegTimer = setInterval(tick, 120)
}

/** SSE 与 MQTT 一致：`温度,湿度,光电` */
function tryParseMeasure(text) {
    rawPayload.value = text
    parseError.value = ''
    const parts = text.trim().split(',').map((s) => s.trim())
    if (parts.length !== 3) {
        parseError.value = '需要三列 CSV：温度,湿度,光电'
        sample.value = null
        return
    }
    const t = Number(parts[0])
    const h = Number(parts[1])
    const p = Number(parts[2])
    if (![t, h, p].every((n) => Number.isFinite(n))) {
        parseError.value = '三列须为数字'
        sample.value = null
        return
    }
    sample.value = { temperature: t, humidity: h, photoelectric: p }
}

function setupHls() {
    hlsError.value = ''
    const video = hlsVideoEl.value
    const src = hlsUrl.value
    if (!video || !src) return

    if (Hls.isSupported()) {
        streamMode.value = 'hls'
        hlsPlayer = new Hls({
            enableWorker: true,
            lowLatencyMode: true,
        })
        hlsPlayer.loadSource(src)
        hlsPlayer.attachMedia(video)
        hlsPlayer.on(Hls.Events.ERROR, (_, data) => {
            if (data.fatal) {
                hlsError.value = `[HLS] ${data.type} ${data.details || ''}（确认 ffmpeg 已写 ${src}）`
            }
        })
    } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
        streamMode.value = 'hls'
        video.src = src
    } else {
        streamMode.value = 'mjpeg'
    }
}

onMounted(async () => {
    tauriEnv.value = isTauri()
    if (tauriEnv.value) {
        try {
            const paths = await invoke('media_asset_paths')
            hlsUrl.value = convertFileSrc(paths.hls_index)
            mjpegSnapUrl.value = convertFileSrc(paths.mjpeg_last)
            mjpegSrc.value = `${mjpegSnapUrl.value}?t=0`
        } catch (e) {
            hlsError.value = `无法取得媒体路径: ${e}`
        }
    }

    es = new EventSource(sseUrl)
    es.onmessage = (ev) => {
        sseError.value = ''
        tryParseMeasure(ev.data)
    }
    es.onerror = () => {
        sseError.value = `SSE 连接失败或已断开（${sseUrl}）`
    }

    await nextTick()
    if (tauriEnv.value) {
        setupHls()
        if (streamMode.value === 'mjpeg') {
            startMjpegPoll()
        }
    }
})

onUnmounted(() => {
    es?.close()
    hlsPlayer?.destroy()
    hlsPlayer = null
    clearInterval(mjpegTimer)
    mjpegTimer = null
})
</script>

<style scoped>
.card {
    margin-bottom: 1.5rem;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 8px;
}
.err {
    color: #c00;
}
.raw {
    opacity: 0.85;
    font-size: 0.85em;
    white-space: pre-wrap;
    word-break: break-all;
}
.hls-video {
    width: 100%;
    max-width: 720px;
    margin-top: 0.5rem;
    background: #111;
}
.mjpeg-preview {
    display: block;
    object-fit: contain;
    min-height: 200px;
}
</style>
