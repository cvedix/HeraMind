#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_BASE="${HERAMIND_API_BASE:-http://127.0.0.1:9375/api}"
API_BASE="${API_BASE%/}"

BROKER_ID="${BODYCAM_BROKER_ID:-bodycam-events}"
BROKER_HOST="${BODYCAM_MQTT_HOST:-192.168.1.239}"
BROKER_PORT="${BODYCAM_MQTT_PORT:-1883}"
BROKER_TOPIC="${BODYCAM_MQTT_TOPIC:-events}"
HLS_URL="${BODYCAM_HLS_URL:-http://192.168.1.239:3546/hls/3e0d9c56-3f25-c54b-2933-ba0e23720611.m3u8}"
DEVICE_ID="${BODYCAM_DEVICE_ID:-G25A06689}"
DEVICE_NAME="${BODYCAM_DEVICE_NAME:-BodyCam G25A06689}"
DEVICE_TYPE="bodycam_wearable_video_recorder"
TRANSFORM_NAME="BodyCam Event Attribute Normalizer"
DASHBOARD_NAME="BodyCam Analytics - G25A06689"

for command_name in curl jq; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

CURL_AUTH=()
if [[ -n "${HERAMIND_API_KEY:-}" ]]; then
  CURL_AUTH=(-H "X-API-Key: ${HERAMIND_API_KEY}")
elif [[ -n "${HERAMIND_TOKEN:-}" ]]; then
  CURL_AUTH=(-H "Authorization: Bearer ${HERAMIND_TOKEN}")
else
  echo "Set HERAMIND_API_KEY or HERAMIND_TOKEN before running this script." >&2
  exit 1
fi

request() {
  local method="$1"
  local path="$2"
  local body_file="${3:-}"
  local response_file
  local status

  response_file="$(mktemp)"
  if [[ -n "$body_file" ]]; then
    status="$(curl -sS -o "$response_file" -w '%{http_code}' \
      -X "$method" \
      "${CURL_AUTH[@]}" \
      -H 'Content-Type: application/json' \
      --data-binary "@${body_file}" \
      "${API_BASE}${path}")"
  else
    status="$(curl -sS -o "$response_file" -w '%{http_code}' \
      -X "$method" \
      "${CURL_AUTH[@]}" \
      "${API_BASE}${path}")"
  fi

  if (( status < 200 || status >= 300 )); then
    echo "HeraMind API ${method} ${path} failed with HTTP ${status}:" >&2
    jq . "$response_file" >&2 2>/dev/null || sed -n '1,120p' "$response_file" >&2
    rm -f "$response_file"
    return 1
  fi

  jq -c . "$response_file"
  rm -f "$response_file"
}

make_temp_json() {
  mktemp
}

echo "Checking HeraMind at ${API_BASE}..."
request GET "/settings/timezone" >/dev/null

echo "Registering BodyCam device type..."
request POST "/device-types" "${SCRIPT_DIR}/device-type.json" >/dev/null

device_payload="$(make_temp_json)"
jq -n \
  --arg device_type "$DEVICE_TYPE" \
  --arg device_id "$DEVICE_ID" \
  --arg name "$DEVICE_NAME" \
  --arg topic "$BROKER_TOPIC" \
  --arg broker_id "$BROKER_ID" \
  '{
    device_type: $device_type,
    device_id: $device_id,
    name: $name,
    adapter_type: "mqtt",
    connection_config: {
      telemetry_topic: $topic,
      broker_id: $broker_id,
      serial_number: "G25A06689",
      ip_address: "192.168.1.185",
      mac_address: "40-D9-5A-EA-D1-57",
      device_class: "Wearable Video Recorder"
    }
  }' > "$device_payload"

devices="$(request GET "/devices?limit=1000")"
if jq -e --arg id "$DEVICE_ID" \
  '(.data.devices // .devices // []) | any((.device_id // .id) == $id)' \
  >/dev/null <<<"$devices"; then
  echo "Updating device ${DEVICE_ID}..."
  update_device_payload="$(make_temp_json)"
  jq '{name, adapter_type, connection_config}' "$device_payload" > "$update_device_payload"
  request PUT "/devices/${DEVICE_ID}" "$update_device_payload" >/dev/null
  rm -f "$update_device_payload"
else
  echo "Creating device ${DEVICE_ID}..."
  request POST "/devices" "$device_payload" >/dev/null
fi
rm -f "$device_payload"

# Connect the external broker only after the device exists. The `events` topic
# can be high-frequency; this ordering prevents the auto-onboarding path from
# creating a transient unknown-device draft before the explicit topic mapping
# has been registered.
broker_payload="$(make_temp_json)"
jq -n \
  --arg id "$BROKER_ID" \
  --arg name "BodyCam MQTT Events" \
  --arg broker "$BROKER_HOST" \
  --argjson port "$BROKER_PORT" \
  --arg topic "$BROKER_TOPIC" \
  --arg username "${BODYCAM_MQTT_USERNAME:-}" \
  --arg password "${BODYCAM_MQTT_PASSWORD:-}" \
  '{
    id: $id,
    name: $name,
    broker: $broker,
    port: $port,
    tls: false,
    enabled: true,
    client_id: "heramind-bodycam-events",
    subscribe_topics: [$topic]
  }
  + (if $username == "" then {} else {username: $username} end)
  + (if $password == "" then {} else {password: $password} end)' \
  > "$broker_payload"

brokers="$(request GET "/brokers")"
if jq -e --arg id "$BROKER_ID" \
  '(.data.brokers // .brokers // []) | any(.id == $id)' \
  >/dev/null <<<"$brokers"; then
  echo "Updating external MQTT broker ${BROKER_HOST}:${BROKER_PORT}/${BROKER_TOPIC}..."
  request PUT "/brokers/${BROKER_ID}" "$broker_payload" >/dev/null
else
  echo "Creating external MQTT broker ${BROKER_HOST}:${BROKER_PORT}/${BROKER_TOPIC}..."
  request POST "/brokers" "$broker_payload" >/dev/null
fi
rm -f "$broker_payload"

transform_code="$(<"${SCRIPT_DIR}/transform.js")"
transform_payload="$(make_temp_json)"
jq -n \
  --arg name "$TRANSFORM_NAME" \
  --arg device_id "$DEVICE_ID" \
  --arg code "$transform_code" \
  '{
    name: $name,
    description: "Normalize BodyCam MQTT attributes and crop images into typed dashboard metrics; face_features is intentionally excluded.",
    enabled: true,
    type: "transform",
    definition: {
      scope: {device: $device_id},
      intent: "Normalize BodyCam attributes and crop images for dashboard display",
      js_code: $code,
      output_prefix: "bodycam_event",
      complexity: 2
    }
  }' > "$transform_payload"

automations="$(request GET "/automations?type=transform&limit=1000")"
transform_id="$(jq -r --arg name "$TRANSFORM_NAME" \
  '(.data.automations // .automations // [])[]? | select(.name == $name) | .id' \
  <<<"$automations" | head -n 1)"

if [[ -n "$transform_id" ]]; then
  echo "Updating transform ${transform_id}..."
  request PUT "/automations/${transform_id}" "$transform_payload" >/dev/null
else
  echo "Creating BodyCam event transform..."
  transform_response="$(request POST "/automations" "$transform_payload")"
  transform_id="$(jq -r '.data.automation.id // .automation.id // empty' <<<"$transform_response")"
fi
rm -f "$transform_payload"

if [[ -z "$transform_id" ]]; then
  echo "Could not determine the BodyCam transform ID." >&2
  exit 1
fi

transform_source() {
  local metric="$1"
  jq -n \
    --arg transform_id "$transform_id" \
    --arg metric "bodycam_event.${metric}" \
    '{
      type: "transform",
      sourceId: ("transform:" + $transform_id),
      transformId: $transform_id,
      metricId: $metric,
      timeRange: 1,
      limit: 100,
      aggregateExt: "latest",
      source: "transform",
      id: $transform_id,
      field: $metric,
      mode: "timeseries"
    }'
}

mqtt_message_count_source() {
  jq -n \
    --arg device_id "$DEVICE_ID" \
    '{
      type: "telemetry",
      sourceId: $device_id,
      metricId: "_raw",
      timeRange: 24,
      limit: 1,
      aggregateExt: "count",
      source: "device",
      id: $device_id,
      field: "_raw",
      mode: "timeseries"
    }'
}

crop_image_source() {
  jq -n \
    --arg transform_id "$transform_id" \
    '{
      type: "transform",
      sourceId: ("transform:" + $transform_id),
      transformId: $transform_id,
      metricId: "bodycam_event.crop_image",
      timeRange: 48,
      limit: 200,
      aggregateExt: "raw",
      source: "transform",
      id: $transform_id,
      field: "bodycam_event.crop_image",
      mode: "timeseries",
      transform: "raw",
      params: {includeRawPoints: true, isImage: true}
    }'
}

component() {
  local id="$1"
  local type="$2"
  local title="$3"
  local x="$4"
  local y="$5"
  local w="$6"
  local h="$7"
  local data_source="${8:-null}"
  local config="${9-}"
  if [[ -z "$config" ]]; then
    config='{}'
  fi

  jq -n \
    --arg id "$id" \
    --arg type "$type" \
    --arg title "$title" \
    --argjson x "$x" \
    --argjson y "$y" \
    --argjson w "$w" \
    --argjson h "$h" \
    --argjson data_source "$data_source" \
    --argjson config "$config" \
    '{
      id: $id,
      type: $type,
      title: $title,
      position: {x: $x, y: $y, w: $w, h: $h},
      data_source: $data_source,
      config: $config
    }'
}

led_config='{
  "size": "md",
  "showCard": true,
  "showGlow": true,
  "rules": [
    {"values": "true,1", "state": "warning", "label": "Có"},
    {"values": "false,0", "state": "off", "label": "Không"}
  ],
  "defaultState": "unknown"
}'

components="$(jq -n '[]')"
device_markdown="$(jq -n --arg content \
  "# BodyCam G25A06689\n\n- **Loại:** Wearable Video Recorder\n- **IP:** 192.168.1.185\n- **MAC:** 40-D9-5A-EA-D1-57\n- **MQTT:** ${BROKER_HOST}:${BROKER_PORT} · topic \`${BROKER_TOPIC}\`" \
  '{content: $content, variant: "default"}')"
components="$(jq --argjson item "$(component bodycam-info markdown-display "Thông tin BodyCam" 0 0 8 3 null "$device_markdown")" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component bodycam-mqtt-count value-card "MQTT messages · 24 giờ" 8 0 4 3 "$(mqtt_message_count_source)" '{"size":"lg","variant":"default","showTrend":false,"icon":"radio"}')" '. + [$item]' <<<"$components")"

hls_config="$(jq -n --arg src "$HLS_URL" '{src: $src, type: "hls", autoplay: true, muted: true, controls: true, fit: "contain", rounded: true, showFullscreen: true}')"
crop_config='{"fit":"contain","rounded":true,"zoomable":true,"downloadable":true,"showTitle":true}'
crop_history_config='{"fit":"contain","rounded":true,"showTimestamp":true,"showIndex":true,"limit":200,"timeRange":48}'
components="$(jq --argjson item "$(component bodycam-hls video-display "Live HLS" 0 3 8 5 null "$hls_config")" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component bodycam-crop-latest image-display "Crop event mới nhất" 8 3 4 5 "$(crop_image_source)" "$crop_config")" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component bodycam-crop-history image-history "Lịch sử crop event · 48 giờ" 0 8 12 5 "$(crop_image_source)" "$crop_history_config")" '. + [$item]' <<<"$components")"

for spec in \
  "gender|Giới tính|0|13|3|2" \
  "age|Nhóm tuổi|3|13|3|2" \
  "upper_clothing_color|Màu áo|6|13|3|2" \
  "lower_clothing_color|Màu quần|9|13|3|2"; do
  IFS='|' read -r metric title x y w h <<<"$spec"
  components="$(jq --argjson item "$(component "bodycam-${metric}" value-card "$title" "$x" "$y" "$w" "$h" "$(transform_source "$metric")" '{"size":"md","variant":"default","showTrend":false}')" '. + [$item]' <<<"$components")"
done

for spec in \
  "smoking|Hút thuốc|0|15|3|2" \
  "phone|Dùng điện thoại|3|15|3|2" \
  "face_covered|Che mặt|6|15|3|2" \
  "assisted|Cần hỗ trợ|9|15|3|2" \
  "glasses|Đeo kính|0|17|3|2" \
  "carrying_bag|Mang túi|3|17|3|2" \
  "tattoo|Hình xăm|6|17|3|2"; do
  IFS='|' read -r metric title x y w h <<<"$spec"
  components="$(jq --argjson item "$(component "bodycam-${metric}" led-indicator "$title" "$x" "$y" "$w" "$h" "$(transform_source "$metric")" "$led_config")" '. + [$item]' <<<"$components")"
done

components="$(jq --argjson item "$(component bodycam-event-time value-card "Thời gian sự kiện" 9 17 3 2 "$(transform_source event_time)" '{"size":"sm","variant":"compact","showTrend":false}')" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component bodycam-event-history line-chart "Dòng thời gian sự kiện" 0 19 12 4 "$(transform_source event_timestamp_ms)" '{"showLegend":false,"showGrid":true,"showTooltip":true,"smooth":true,"timeWindow":"last_1hour"}')" '. + [$item]' <<<"$components")"

dashboard_payload="$(make_temp_json)"
jq -n \
  --arg name "$DASHBOARD_NAME" \
  --argjson components "$components" \
  '{
    name: $name,
    layout: {
      columns: 12,
      rows: "auto",
      breakpoints: {lg: 1200, md: 996, sm: 768, xs: 480}
    },
    components: $components
  }' > "$dashboard_payload"

dashboards="$(request GET "/dashboards")"
dashboard_id="$(jq -r --arg name "$DASHBOARD_NAME" \
  '(.data.dashboards // .dashboards // [])[]? | select(.name == $name) | .id' \
  <<<"$dashboards" | head -n 1)"

if [[ -n "$dashboard_id" ]]; then
  echo "Updating dashboard ${dashboard_id}..."
  request PUT "/dashboards/${dashboard_id}" "$dashboard_payload" >/dev/null
else
  echo "Creating BodyCam dashboard..."
  dashboard_response="$(request POST "/dashboards" "$dashboard_payload")"
  dashboard_id="$(jq -r '.data.id // .id // empty' <<<"$dashboard_response")"
fi
rm -f "$dashboard_payload"

echo
echo "BodyCam setup complete."
echo "Device:    ${DEVICE_ID}"
echo "Broker:    mqtt://${BROKER_HOST}:${BROKER_PORT}/${BROKER_TOPIC}"
echo "HLS:       ${HLS_URL}"
echo "Transform: ${transform_id}"
echo "Dashboard: ${dashboard_id:-$DASHBOARD_NAME}"
echo "Web UI:    ${API_BASE%/api}"
