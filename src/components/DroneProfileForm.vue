<template>
  <div class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal">
      <div class="modal-header">
        <h3 class="modal-title">{{ editingName ? 'Edit Drone' : 'Add Drone' }}</h3>
        <button class="modal-close" @click="$emit('cancel')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="3" y1="3" x2="11" y2="11"/>
            <line x1="11" y1="3" x2="3" y2="11"/>
          </svg>
        </button>
      </div>
      <form class="modal-form" @submit.prevent="onSubmit">
        <div class="form-group">
          <label class="form-label">Name</label>
          <input class="form-input" v-model="form.name" required :disabled="!!editingName" placeholder="drone-1" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">MQTT Host</label>
            <input class="form-input" v-model="form.mqtt_broker_host" required placeholder="127.0.0.1" />
          </div>
          <div class="form-group form-group-sm">
            <label class="form-label">Port</label>
            <input class="form-input" v-model.number="form.mqtt_broker_port" type="number" required placeholder="1883" />
          </div>
        </div>
        <div class="form-group">
          <label class="form-label">MQTT Topic</label>
          <input class="form-input" v-model="form.mqtt_topic" required placeholder="measure/data" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">MQTT Username</label>
            <input class="form-input" v-model="form.mqtt_username" placeholder="(optional)" />
          </div>
          <div class="form-group">
            <label class="form-label">MQTT Password</label>
            <input class="form-input" v-model="form.mqtt_password" type="password" placeholder="(optional)" />
          </div>
        </div>
        <div class="form-group">
          <label class="form-label">RTSP URL</label>
          <input class="form-input" v-model="form.rtsp_url" placeholder="rtsp://host:port/stream" />
        </div>
        <div class="form-group form-group-checkbox">
          <label>
            <input type="checkbox" v-model="form.enabled" />
            <span>Enabled</span>
          </label>
        </div>
        <div class="form-actions">
          <button type="button" class="btn btn-secondary" @click="$emit('cancel')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving">{{ saving ? 'Saving...' : 'Save' }}</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  profile: { type: Object, default: null }
})

const emit = defineEmits(['save', 'cancel'])

const editingName = props.profile?.name || null
const saving = ref(false)

const form = ref({
  name: props.profile?.name || '',
  enabled: props.profile?.enabled ?? true,
  mqtt_broker_host: props.profile?.mqtt_broker_host || '127.0.0.1',
  mqtt_broker_port: props.profile?.mqtt_broker_port || 1883,
  mqtt_topic: props.profile?.mqtt_topic || 'measure/data',
  mqtt_username: props.profile?.mqtt_username || '',
  mqtt_password: props.profile?.mqtt_password || '',
  rtsp_url: props.profile?.rtsp_url || ''
})

async function onSubmit() {
  if (!form.value.name.trim()) return
  saving.value = true
  try {
    await emit('save', {
      name: form.value.name.trim(),
      enabled: form.value.enabled,
      mqtt_broker_host: form.value.mqtt_broker_host.trim(),
      mqtt_broker_port: Number(form.value.mqtt_broker_port) || 1883,
      mqtt_topic: form.value.mqtt_topic.trim(),
      mqtt_username: form.value.mqtt_username.trim() || null,
      mqtt_password: form.value.mqtt_password || null,
      rtsp_url: form.value.rtsp_url.trim()
    })
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  width: 440px;
  max-width: 90vw;
  max-height: 85vh;
  overflow-y: auto;
  background: #16161e;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.modal-title {
  font-size: 16px;
  font-weight: 700;
  color: #e2e8f0;
  margin: 0;
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: #94a3b8;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.modal-close:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.modal-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group-sm {
  max-width: 100px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

.form-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.form-input {
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #e2e8f0;
  font-size: 13px;
  font-family: var(--font-mono);
  outline: none;
  transition: border-color 0.15s ease;
}

.form-input:focus {
  border-color: rgba(59, 130, 246, 0.5);
}

.form-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-input::placeholder {
  color: #334155;
}

.form-group-checkbox label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #94a3b8;
  cursor: pointer;
}

.form-group-checkbox input[type="checkbox"] {
  accent-color: #3b82f6;
  width: 16px;
  height: 16px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}

.btn {
  padding: 8px 20px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.05);
  color: #94a3b8;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
}

.btn-primary {
  background: #3b82f6;
  color: #fff;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
