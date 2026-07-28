#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_BASE="${HERAMIND_API_BASE:-http://127.0.0.1:9375/api}"
API_BASE="${API_BASE%/}"

BROKER_ID="${HERACAM_BROKER_ID:-heracam-rv1126b-demo}"
BROKER_HOST="${HERACAM_MQTT_HOST:-192.168.1.239}"
BROKER_PORT="${HERACAM_MQTT_PORT:-1883}"
BROKER_TOPIC="${HERACAM_MQTT_TOPIC:-Demo}"
DEVICE_ID="${HERACAM_DEVICE_ID:-HERACAM-RV1126B-DEMO}"
DEVICE_NAME="${HERACAM_DEVICE_NAME:-HERACAM RV1126B Demo}"
DEVICE_TYPE="heracam_rv1126b_vehicle_analytics"
TRANSFORM_NAME="HERACAM RV1126B Vehicle Event Normalizer"
DASHBOARD_NAME="HERACAM RV1126B - Báo cáo phương tiện"

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

echo "Registering HERACAM RV1126B device type..."
request POST "/device-types" "${SCRIPT_DIR}/device-type.json" >/dev/null

device_payload="$(make_temp_json)"
jq -n \
  --arg device_type "$DEVICE_TYPE" \
  --arg device_id "$DEVICE_ID" \
  --arg name "$DEVICE_NAME" \
  --arg topic "$BROKER_TOPIC" \
  --arg broker_id "$BROKER_ID" \
  --arg broker_host "$BROKER_HOST" \
  --argjson broker_port "$BROKER_PORT" \
  '{
    device_type: $device_type,
    device_id: $device_id,
    name: $name,
    adapter_type: "mqtt",
    connection_config: {
      telemetry_topic: $topic,
      broker_id: $broker_id,
      manufacturer: "HERACAM",
      model: "RV1126B",
      device_class: "Vehicle Analytics Camera",
      simulated: true,
      mqtt_host: $broker_host,
      mqtt_port: $broker_port
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

broker_payload="$(make_temp_json)"
jq -n \
  --arg id "$BROKER_ID" \
  --arg name "HERACAM RV1126B Demo MQTT" \
  --arg broker "$BROKER_HOST" \
  --argjson port "$BROKER_PORT" \
  --arg topic "$BROKER_TOPIC" \
  --arg username "${HERACAM_MQTT_USERNAME:-}" \
  --arg password "${HERACAM_MQTT_PASSWORD:-}" \
  '{
    id: $id,
    name: $name,
    broker: $broker,
    port: $port,
    tls: false,
    enabled: true,
    client_id: "heramind-heracam-rv1126b-demo",
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
    description: "Normalize HERACAM RV1126B Demo MQTT vehicle events, crop images, and vehicle attributes into dashboard metrics.",
    enabled: true,
    type: "transform",
    definition: {
      scope: {device: $device_id},
      intent: "Build vehicle reporting metrics from HERACAM RV1126B Demo MQTT events",
      js_code: $code,
      output_prefix: "heracam_vehicle",
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
  echo "Creating HERACAM vehicle transform..."
  transform_response="$(request POST "/automations" "$transform_payload")"
  transform_id="$(jq -r '.data.automation.id // .automation.id // empty' <<<"$transform_response")"
fi
rm -f "$transform_payload"

if [[ -z "$transform_id" ]]; then
  echo "Could not determine the HERACAM transform ID." >&2
  exit 1
fi

transform_source() {
  local metric="$1"
  local aggregate="${2:-latest}"
  local time_range="${3:-1}"
  local limit="${4:-100}"
  jq -n \
    --arg transform_id "$transform_id" \
    --arg metric "heracam_vehicle.${metric}" \
    --arg aggregate "$aggregate" \
    --argjson time_range "$time_range" \
    --argjson limit "$limit" \
    '{
      type: "transform",
      sourceId: ("transform:" + $transform_id),
      transformId: $transform_id,
      metricId: $metric,
      timeRange: $time_range,
      limit: $limit,
      aggregateExt: $aggregate,
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

vehicle_trend_source() {
  jq -n \
    --arg device_id "$DEVICE_ID" \
    '{
      type: "telemetry",
      sourceId: $device_id,
      metricId: "heracam_vehicle.vehicle_seen_numeric",
      timeRange: 6,
      limit: 3000,
      aggregateExt: "raw",
      source: "device",
      id: $device_id,
      field: "heracam_vehicle.vehicle_seen_numeric",
      mode: "timeseries",
      transform: "raw",
      params: {includeRawPoints: true},
      timeWindow: {type: "last_6hours"}
    }'
}

crop_image_source() {
  jq -n \
    --arg transform_id "$transform_id" \
    '{
      type: "transform",
      sourceId: ("transform:" + $transform_id),
      transformId: $transform_id,
      metricId: "heracam_vehicle.crop_image",
      timeRange: 48,
      limit: 200,
      aggregateExt: "raw",
      source: "transform",
      id: $transform_id,
      field: "heracam_vehicle.crop_image",
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

components="$(jq -n '[]')"
device_info_content="$(printf '# HERACAM RV1126B\n\n- **Loại:** Vehicle Analytics Camera\n- **Chế độ:** mô phỏng thiết bị HERACAM RV1126B\n- **MQTT:** %s:%s · topic `%s`\n- **Event chính:** `event-line-crossing`, `crop`, `attribute`\n- **Attribute đã thấy:** `vehicle_class`, `vehicle_color`' "$BROKER_HOST" "$BROKER_PORT" "$BROKER_TOPIC")"
device_markdown="$(jq -n --arg content "$device_info_content" '{content: $content, variant: "default"}')"

components="$(jq --argjson item "$(component heracam-info markdown-display "Thông tin HERACAM" 0 0 6 3 null "$device_markdown")" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component heracam-total-vehicles value-card "Tổng phương tiện · 24 giờ" 6 0 3 3 "$(transform_source vehicle_seen count 24 1)" '{"size":"lg","variant":"default","showTrend":true,"icon":"car"}')" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component heracam-mqtt-count value-card "MQTT messages · 24 giờ" 9 0 3 3 "$(mqtt_message_count_source)" '{"size":"lg","variant":"default","showTrend":false,"icon":"radio"}')" '. + [$item]' <<<"$components")"

crop_config='{"fit":"contain","rounded":true,"zoomable":true,"downloadable":true,"showTitle":true}'
crop_history_config='{"fit":"contain","rounded":true,"showTimestamp":true,"showIndex":true,"limit":200,"timeRange":48}'
components="$(jq --argjson item "$(component heracam-crop-latest image-display "Ảnh event mới nhất" 0 3 5 5 "$(crop_image_source)" "$crop_config")" '. + [$item]' <<<"$components")"
components="$(jq --argjson item "$(component heracam-vehicle-trend line-chart "Số lượng phương tiện mỗi phút · 6 giờ" 5 3 7 5 "$(vehicle_trend_source)" '{"showLegend":false,"showGrid":true,"showTooltip":true,"smooth":false,"fillArea":true,"size":"lg","dataMapping":{"timeAggregate":"1m","aggregate":"sum","fillMissingBuckets":true}}')" '. + [$item]' <<<"$components")"

for spec in \
  "vehicle_color|Màu sắc|0|8|3|2|palette" \
  "vehicle_class|Loại phương tiện|3|8|3|2|car" \
  "crossing_direction|Hướng di chuyển|6|8|3|2|arrow-up-down" \
  "crop_confidence|Độ tin cậy|9|8|3|2|scan-search" \
  "event_time|Thời gian event|0|10|3|2|clock"; do
  IFS='|' read -r metric title x y w h icon <<<"$spec"
  components="$(jq --argjson item "$(component "heracam-${metric}" value-card "$title" "$x" "$y" "$w" "$h" "$(transform_source "$metric")" "{\"size\":\"md\",\"variant\":\"default\",\"showTrend\":false,\"icon\":\"${icon}\"}")" '. + [$item]' <<<"$components")"
done

components="$(jq --argjson item "$(component heracam-crop-history image-history "Lịch sử hình ảnh event · 48 giờ" 0 12 12 5 "$(crop_image_source)" "$crop_history_config")" '. + [$item]' <<<"$components")"

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
  echo "Creating HERACAM dashboard..."
  dashboard_response="$(request POST "/dashboards" "$dashboard_payload")"
  dashboard_id="$(jq -r '.data.id // .id // empty' <<<"$dashboard_response")"
fi
rm -f "$dashboard_payload"

echo
echo "HERACAM RV1126B setup complete."
echo "Device:    ${DEVICE_ID}"
echo "Broker:    mqtt://${BROKER_HOST}:${BROKER_PORT}/${BROKER_TOPIC}"
echo "Transform: ${transform_id}"
echo "Dashboard: ${dashboard_id:-$DASHBOARD_NAME}"
echo "Web UI:    ${API_BASE%/api}"
