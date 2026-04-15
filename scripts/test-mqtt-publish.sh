#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ROOT}/.env"

get_env() {
  local key="$1"
  grep "^${key}=" "$ENV_FILE" | head -1 | cut -d= -f2-
}

if [[ ! -f "$ENV_FILE" ]]; then
  echo "缺少 ${ENV_FILE}" >&2
  exit 1
fi

if ! command -v mosquitto_pub &>/dev/null; then
  echo "未找到 mosquitto_pub，请先安装 mosquitto-clients（见本脚本头部注释）" >&2
  exit 1
fi

HOST="$(get_env MQTT_BROKER_HOST)"
PORT="$(get_env MQTT_BROKER_PORT)"
TOPIC="$(get_env MQTT_MEASURE_TOPIC)"
PAYLOAD='23.5,60.0,0.12'

echo "发布到 ${HOST}:${PORT} 主题 [${TOPIC}]"
mosquitto_pub -h "$HOST" -p "$PORT" -t "$TOPIC" -m "$PAYLOAD"
echo "已发送: ${PAYLOAD}"
