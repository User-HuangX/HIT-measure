<template>
    <div>
        <h1>HIT-measure</h1>

        <section class="card">
            <h2>无人机视角</h2>
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
const hlsUrl = `${streamOrigin()}/hls/index.m3u8`
const mjpegSnapUrl = `${streamOrigin()}/mjpeg/last.jpg`

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
const mjpegSrc = ref(`${mjpegSnapUrl}?t=0`)
let hlsPlayer = null
let mjpegTimer = null

const sample = ref(null)
const rawPayload = ref('')
const parseError = ref('')
const sseError = ref('')
let es = null

function onMjpegImgError() {
    if (streamMode.value !== 'mjpeg') return
    hlsError.value = `MJPEG 无法加载（${mjpegSnapUrl}）。若刚启动请稍等；并确认 RTSP_RELAY_ENABLED、ffmpeg 与 RTSP 地址。`
}

function startMjpegPoll() {
    clearInterval(mjpegTimer)
    hlsError.value = ''
    const tick = () => {
        mjpegSrc.value = `${mjpegSnapUrl}?t=${Date.now()}`
    }
    tick()
    mjpegTimer = setInterval(tick, 120)
}

function tryParseMeasure(text) {
    rawPayload.value = text
    parseError.value = ''
    try {
        const o = JSON.parse(text)
        if (
            typeof o.temperature === 'number' &&
            typeof o.humidity === 'number' &&
            typeof o.photoelectric === 'number'
        ) {
            sample.value = {
                temperature: o.temperature,
                humidity: o.humidity,
                photoelectric: o.photoelectric,
            }
        } else {
            parseError.value =
                'JSON 字段不完整（需要 temperature / humidity / photoelectric 数字）'
            sample.value = null
        }
    } catch {
        parseError.value = '不是合法 JSON'
        sample.value = null
    }
}

function setupHls() {
    hlsError.value = ''
    const video = hlsVideoEl.value
    if (!video) return

    if (Hls.isSupported()) {
        streamMode.value = 'hls'
        hlsPlayer = new Hls({
            enableWorker: true,
            lowLatencyMode: true,
        })
        hlsPlayer.loadSource(hlsUrl)
        hlsPlayer.attachMedia(video)
        hlsPlayer.on(Hls.Events.ERROR, (_, data) => {
            if (data.fatal) {
                hlsError.value = `[HLS] ${data.type} ${data.details || ''}（确认 ffmpeg 已启动且 ${hlsUrl} 可访问）`
            }
        })
    } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
        streamMode.value = 'hls'
        video.src = hlsUrl
    } else {
        streamMode.value = 'mjpeg'
    }
}

onMounted(() => {
    es = new EventSource(sseUrl)
    es.onmessage = (ev) => {
        sseError.value = ''
        tryParseMeasure(ev.data)
    }
    es.onerror = () => {
        sseError.value = `SSE 连接失败或已断开（${sseUrl}）`
    }

    nextTick(() => {
        setupHls()
        if (streamMode.value === 'mjpeg') {
            startMjpegPoll()
        }
    })
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
.hint {
    font-size: 0.9em;
    color: #555;
}
.err {
    color: #c00;
}
.raw,
.url {
    opacity: 0.85;
    font-size: 0.85em;
    white-space: pre-wrap;
    word-break: break-all;
}
.mono {
    font-family: ui-monospace, monospace;
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
