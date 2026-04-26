#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${ROOT}/.env"

get_env() {
  local key="$1"
  awk -F= -v k="$key" '$1==k {print substr($0, index($0, "=")+1); exit}' "$ENV_FILE"
}

require_env() {
  local key="$1"
  local value
  value="$(get_env "$key")"
  if [[ -z "$value" ]]; then
    echo "缺少配置 ${key}（请检查 ${ENV_FILE}）" >&2
    exit 1
  fi
  printf '%s' "$value"
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
USERNAME="$(get_env MQTT_USERNAME)"
PASSWORD="$(get_env MQTT_PASSWORD)"
QOS="$(get_env MQTT_QOS)"
PAYLOAD='23.5,60.0,0.12'

HOST="$(require_env MQTT_BROKER_HOST)"
PORT="$(require_env MQTT_BROKER_PORT)"
TOPIC="$(require_env MQTT_MEASURE_TOPIC)"
QOS="${QOS:-0}"

if ! [[ "$PORT" =~ ^[0-9]+$ ]]; then
  echo "MQTT_BROKER_PORT 必须是数字，当前值: ${PORT}" >&2
  exit 1
fi

if command -v timeout >/dev/null 2>&1; then
  if ! timeout 2 bash -c "cat < /dev/null > /dev/tcp/${HOST}/${PORT}" 2>/dev/null; then
    echo "MQTT Broker 不可达: ${HOST}:${PORT}（请先启动 broker）" >&2
    exit 1
  fi
fi

echo "发布到 ${HOST}:${PORT} 主题 [${TOPIC}]"
cmd=(mosquitto_pub -h "$HOST" -p "$PORT" -t "$TOPIC" -q "$QOS" -m "$PAYLOAD")
if [[ -n "$USERNAME" ]]; then
  cmd+=(-u "$USERNAME")
fi
if [[ -n "$PASSWORD" ]]; then
  cmd+=(-P "$PASSWORD")
fi
"${cmd[@]}"
echo "已发送: ${PAYLOAD}"
