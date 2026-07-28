const raw = input_raw._raw
  || (input_raw.values && input_raw.values._raw)
  || input;

let payload = raw;
if (typeof payload === 'string') {
  try {
    payload = JSON.parse(payload);
  } catch (_) {
    payload = [];
  }
}

const records = Array.isArray(payload)
  ? payload
  : Array.isArray(payload && payload.events)
    ? payload.events
    : Array.isArray(payload && payload.data)
      ? payload.data
      : payload && typeof payload === 'object'
        ? [payload]
        : [];

function asNumber(value, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function cleanString(value, fallback = '') {
  if (value === null || value === undefined) return fallback;
  const text = String(value).trim();
  return text || fallback;
}

function normalizeImage(value) {
  if (typeof value === 'string' && value.trim()) {
    const text = value.trim();
    if (/^\/9j\//.test(text)) return `data:image/jpeg;base64,${text}`;
    if (/^iVBOR/.test(text)) return `data:image/png;base64,${text}`;
    if (/^(data:image\/|https?:\/\/|\/)/i.test(text)) return text;
    return text;
  }
  if (!value || typeof value !== 'object') return '';

  for (const key of ['url', 'src', 'image_url', 'imageUrl', 'image', 'data', 'base64', 'value', 'path']) {
    const nested = value[key];
    const image = normalizeImage(nested);
    if (image) return image;
  }
  return '';
}

function getLocation(item) {
  const location = item && item.location && typeof item.location === 'object'
    ? item.location
    : {};
  return {
    x: asNumber(location.x),
    y: asNumber(location.y),
    width: asNumber(location.width),
    height: asNumber(location.height)
  };
}

const vehicleEvents = records.filter(item =>
  item &&
  typeof item === 'object' &&
  (String(item.$id || '').startsWith('event-') || cleanString(item.object_class).toLowerCase() === 'vehicle')
);

const eventIds = new Set(vehicleEvents.map(item => cleanString(item.event_id)).filter(Boolean));
const attributes = {};
for (const item of records) {
  if (!item || typeof item !== 'object' || item.$id !== 'attribute') continue;
  const name = cleanString(item.name);
  if (!name) continue;
  attributes[name] = item.value;
}

const crops = records.filter(item =>
  item &&
  typeof item === 'object' &&
  item.$id === 'crop' &&
  normalizeImage(item.image || item)
);

crops.sort((left, right) => {
  const leftMatchesEvent = eventIds.has(cleanString(left.ref_event_id)) ? 1 : 0;
  const rightMatchesEvent = eventIds.has(cleanString(right.ref_event_id)) ? 1 : 0;
  if (leftMatchesEvent !== rightMatchesEvent) return rightMatchesEvent - leftMatchesEvent;
  return asNumber(right.confidence) - asNumber(left.confidence);
});

const primaryEvent = vehicleEvents[0] || records.find(item => item && typeof item === 'object') || {};
const bestCrop = crops[0] || null;
const metadataSource = primaryEvent || bestCrop || {};
const cropImage = bestCrop ? normalizeImage(bestCrop.image || bestCrop) : '';
const objectClass = cleanString(attributes.vehicle_class, cleanString(primaryEvent.object_class, 'Vehicle'));
const vehicleColor = cleanString(attributes.vehicle_color, 'unknown');
const location = getLocation(primaryEvent.location ? primaryEvent : bestCrop || {});

const result = {
  device_name: 'HERACAM RV1126B',
  device_model: 'RV1126B',
  device_vendor: 'HERACAM',
  mqtt_topic: 'Demo',
  event_type: cleanString(primaryEvent.$id || bestCrop?.$id, 'unknown'),
  event_id: cleanString(primaryEvent.event_id || bestCrop?.ref_event_id),
  instance_id: cleanString(metadataSource.instance_id),
  tracking_id: cleanString(metadataSource.ref_tracking_id),
  event_timestamp_ms: asNumber(metadataSource.event_timestamp_ms),
  system_timestamp: asNumber(metadataSource.system_timestamp),
  event_time: cleanString(metadataSource.system_datetime),
  vehicle_class: objectClass,
  vehicle_color: vehicleColor,
  crossing_direction: cleanString(primaryEvent.crossing_direction, 'unknown'),
  tripwire_name: cleanString(primaryEvent.tripwire_name, 'unknown'),
  tripwire_id: cleanString(primaryEvent.tripwire_id),
  crop_count: crops.length,
  attribute_count: Object.keys(attributes).length,
  has_crop: Boolean(cropImage),
  bbox_x: location.x,
  bbox_y: location.y,
  bbox_width: location.width,
  bbox_height: location.height
};

if (vehicleEvents.length > 0) {
  result.vehicle_seen = true;
  result.vehicle_seen_numeric = 1;
  result.vehicle_count_in_message = vehicleEvents.length;
}

if (bestCrop) {
  result.crop_confidence = asNumber(bestCrop.confidence);
  result.crop_timestamp_ms = asNumber(bestCrop.crop_timestamp_ms);
  result.crop_event_id = cleanString(bestCrop.ref_event_id);
}

if (cropImage) {
  result.crop_image = cropImage;
}

return result;
