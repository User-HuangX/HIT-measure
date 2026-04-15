<template>
    <div>
        <h1>HIT-measure</h1>
        <p v-if="sseError">{{ sseError }}</p>
        <pre v-else-if="lastJson">{{ lastJson }}</pre>
        <p v-else>等待 SSE（{{ sseUrl }}）…</p>
    </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

const sseUrl = 'http://127.0.0.1:5888/events'
const lastJson = ref('')
const sseError = ref('')
let es = null

onMounted(() => {
    es = new EventSource(sseUrl)
    es.onmessage = (ev) => {
        lastJson.value = ev.data
        sseError.value = ''
    }
    es.onerror = () => {
        sseError.value = 'SSE 连接失败或已断开（请确认后端已启动且端口 5888 可用）'
    }
})

onUnmounted(() => {
    es?.close()
})
</script>


