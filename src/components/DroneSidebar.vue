<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Drones</span>
      <button class="add-btn" @click="$emit('add')" title="Add drone">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <line x1="7" y1="2" x2="7" y2="12"/>
          <line x1="2" y1="7" x2="12" y2="7"/>
        </svg>
      </button>
    </div>
    <ul class="drone-list">
      <li
        v-for="profile in profiles"
        :key="profile.name"
        class="drone-item"
        :class="{ active: profile.name === activeDrone, disabled: !profile.enabled }"
        @click="switchTo(profile.name)"
      >
        <div class="drone-info">
          <span class="drone-name" :title="profile.name">{{ profile.name }}</span>
          <span class="drone-status" :class="profile.enabled ? 'status-on' : 'status-off'">
            {{ profile.enabled ? 'ON' : 'OFF' }}
          </span>
        </div>
        <div class="drone-actions">
          <button class="edit-btn" @click.stop="$emit('edit', profile)" title="Edit">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="delete-btn" @click.stop="$emit('delete', profile.name)" title="Delete">
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
              <line x1="2" y1="2" x2="8" y2="8"/>
              <line x1="8" y1="2" x2="2" y2="8"/>
            </svg>
          </button>
        </div>
      </li>
    </ul>
    <div v-if="profiles.length === 0" class="empty-hint">
      No drones configured. Click + to add one.
    </div>
    <div class="sidebar-footer">
      <button class="settings-btn" @click="$emit('settings')" title="Global Settings">
        <Settings :size="14" />
        <span>Settings</span>
      </button>
    </div>
  </aside>
</template>

<script setup>
import { Settings } from 'lucide-vue-next'
const emit = defineEmits(['add', 'delete', 'switch', 'settings', 'edit'])

defineProps({
  profiles: { type: Array, default: () => [] },
  activeDrone: { type: String, default: null }
})

function switchTo(name) {
  emit('switch', name)
}
</script>

<style scoped>
.sidebar {
  position: relative;
  z-index: 1;
  width: 220px;
  min-width: 220px;
  background: rgba(255, 255, 255, 0.02);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.sidebar-title {
  font-size: 13px;
  font-weight: 700;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.add-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.03);
  color: #94a3b8;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.add-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
  border-color: rgba(255, 255, 255, 0.2);
}

.drone-list {
  list-style: none;
  margin: 0;
  padding: 8px;
  flex: 1;
  overflow-y: auto;
}

.drone-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  margin-bottom: 4px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid transparent;
}

.drone-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.drone-item.active {
  background: rgba(59, 130, 246, 0.1);
  border-color: rgba(59, 130, 246, 0.2);
}

.drone-item.disabled {
  opacity: 0.5;
}

.drone-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.drone-name {
  font-size: 13px;
  font-weight: 500;
  color: #e2e8f0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.drone-status {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 5px;
  border-radius: 3px;
  letter-spacing: 0.05em;
  flex-shrink: 0;
}

.status-on {
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
}

.status-off {
  background: rgba(100, 116, 139, 0.15);
  color: #64748b;
}

.drone-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.delete-btn,
.edit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: #475569;
  cursor: pointer;
  border-radius: 4px;
  opacity: 0;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.drone-item:hover .delete-btn,
.drone-item:hover .edit-btn {
  opacity: 1;
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.edit-btn:hover {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
}

.empty-hint {
  padding: 20px 16px;
  font-size: 12px;
  color: #475569;
  text-align: center;
  line-height: 1.5;
}

.sidebar-footer {
  padding: 12px 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  color: #64748b;
  cursor: pointer;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.settings-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #94a3b8;
  border-color: rgba(255, 255, 255, 0.12);
}
</style>
