<template>
    <div>
        <h1>HIT-measure</h1>

        <section class="card">
            <h2>视频（RTSP → 预览）</h2>
            <p v-if="!tauriEnv" class="err">请在 Tauri 窗口中运行。</p>
            <template v-else>
                <img
                    class="preview"
                    :src="mjpegSrc"
                    alt="预览"
                    @error="onImgError"
                />
                <p v-if="videoErr" class="err">{{ videoErr }}</p>
            </template>
        </section>

        <section class="card">
            <h2>测量（MQTT）</h2>
            <p v-if="mqttErr" class="err">{{ mqttErr }}</p>
            <div v-else-if="sample">
                <p>温度：{{ sample.temperature }}</p>
                <p>湿度：{{ sample.humidity }}</p>
                <p>光电：{{ sample.photoelectric }}</p>
            </div>
            <p v-else>等待 MQTT 数据…</p>
        </section>
    </div>
</template>

<script setup>
import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref } from 'vue'

const tauriEnv = ref(false)
const mjpegSrc = ref('')
const videoErr = ref('')
const mqttErr = ref('')
const sample = ref(null)
let mjpegTimer = null
let unlistenMeasure = () => {}

function onImgError() {
    videoErr.value = '预览图加载失败（等待 ffmpeg 首帧或检查 RTSP_RELAY_*）'
}

function startMjpegPoll(base) {
    clearInterval(mjpegTimer)
    videoErr.value = ''
    const tick = () => {
        if (!base) return
        mjpegSrc.value = `${base}?t=${Date.now()}`
    }
    tick()
    mjpegTimer = setInterval(tick, 120)
}

onMounted(async () => {
    tauriEnv.value = isTauri()
    if (!tauriEnv.value) return

    try {
        const { last_jpeg } = await invoke('mjpeg_asset_path')
        const url = convertFileSrc(last_jpeg)
        startMjpegPoll(url)
    } catch (e) {
        videoErr.value = String(e)
    }

    try {
        unlistenMeasure = await listen('measure', (ev) => {
            mqttErr.value = ''
            const p = ev.payload
            sample.value = {
                temperature: p.temperature,
                humidity: p.humidity,
                photoelectric: p.photoelectric,
            }
        })
    } catch (e) {
        mqttErr.value = `订阅事件失败: ${e}`
    }
})

onUnmounted(() => {
    clearInterval(mjpegTimer)
    mjpegTimer = null
    unlistenMeasure()
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
.preview {
    display: block;
    width: 100%;
    max-width: 720px;
    margin-top: 0.5rem;
    min-height: 200px;
    background: #111;
    object-fit: contain;
}
</style>
