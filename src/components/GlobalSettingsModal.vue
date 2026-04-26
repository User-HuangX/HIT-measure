<template>
  <div class="modal-overlay" @click.self="$emit('cancel')">
    <div class="modal">
      <div class="modal-header">
        <div class="modal-header-left">
          <Settings class="modal-icon" :size="18" />
          <h3 class="modal-title">Global Settings</h3>
        </div>
        <button class="modal-close" @click="$emit('cancel')">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="3" y1="3" x2="11" y2="11"/>
            <line x1="11" y1="3" x2="3" y2="11"/>
          </svg>
        </button>
      </div>
      <form class="modal-form" @submit.prevent="onSubmit">
        <div class="form-group">
          <label class="form-label">DATABASE_URL</label>
          <input class="form-input form-input-mono" v-model="form.database_url"
                 placeholder="postgresql://user:pass@host:5432/dbname" />
          <span class="form-hint">PostgreSQL 连接串，保存后立即生效</span>
        </div>
        <div class="form-group">
          <label class="form-label">Measure Persist Interval</label>
          <div class="form-input-with-suffix">
            <input class="form-input" v-model.number="form.measure_persist_interval_secs"
                   type="number" min="1" max="3600" />
            <span class="input-suffix">seconds</span>
          </div>
          <span class="form-hint">传感器数据写入数据库的间隔时间</span>
        </div>
        <div class="form-group form-group-checkbox">
          <label>
            <input type="checkbox" v-model="form.db_enabled" />
            <span>Enable Database Storage</span>
          </label>
          <span class="form-hint">关闭后停止写入数据库，MQTT 数据仍实时显示</span>
        </div>
        <div v-if="saveError" class="form-error">{{ saveError }}</div>
        <div class="form-actions">
          <button type="button" class="btn btn-secondary" @click="$emit('cancel')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving">{{ saving ? 'Applying...' : 'Apply' }}</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Settings } from 'lucide-vue-next'

const emit = defineEmits(['saved', 'cancel'])

const saving = ref(false)
const saveError = ref('')

const form = ref({
  database_url: '',
  measure_persist_interval_secs: 5,
  db_enabled: true
})

onMounted(async () => {
  try {
    const settings = await invoke('get_global_settings')
    form.value.database_url = settings.database_url || ''
    form.value.measure_persist_interval_secs = settings.measure_persist_interval_secs || 5
    form.value.db_enabled = settings.db_enabled ?? true
  } catch (e) {
    console.error('load global settings:', e)
  }
})

async function onSubmit() {
  saveError.value = ''
  saving.value = true
  try {
    await invoke('update_global_settings', {
      databaseUrl: form.value.database_url.trim(),
      measurePersistIntervalSecs: Number(form.value.measure_persist_interval_secs) || 5,
      dbEnabled: form.value.db_enabled
    })
    emit('saved')
  } catch (e) {
    saveError.value = `保存失败: ${e}`
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
  width: 480px;
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

.modal-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-icon {
  color: #64748b;
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

.form-input::placeholder {
  color: #334155;
}

.form-input-mono {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
}

.form-input-with-suffix {
  display: flex;
  align-items: center;
}

.form-input-with-suffix .form-input {
  border-radius: 8px 0 0 8px;
  border-right: none;
  flex: 1;
}

.input-suffix {
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-left: none;
  border-radius: 0 8px 8px 0;
  color: #64748b;
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.form-hint {
  font-size: 11px;
  color: #475569;
  line-height: 1.4;
}

.form-group-checkbox {
  flex-direction: row;
  align-items: center;
  gap: 8px;
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

.form-group-checkbox .form-hint {
  margin-top: 4px;
}

.form-error {
  font-size: 12px;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 8px;
  padding: 8px 12px;
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
