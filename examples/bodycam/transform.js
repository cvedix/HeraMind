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

const attributes = Array.isArray(payload)
  ? payload
  : Array.isArray(payload && payload.attributes)
    ? payload.attributes
    : Array.isArray(payload && payload.events)
      ? payload.events
      : Array.isArray(payload && payload.data)
        ? payload.data
        : [];

function unwrapImage(value) {
  if (typeof value === 'string' && value.trim()) return value.trim();
  if (!value || typeof value !== 'object') return '';

  for (const key of [
    'url', 'src', 'image_url', 'imageUrl', 'image', 'data',
    'base64', 'value', 'path'
  ]) {
    const nested = value[key];
    if (typeof nested === 'string' && nested.trim()) return nested.trim();
  }
  return '';
}

function findBestCrop(container) {
  if (!Array.isArray(container)) return null;

  const eventIds = new Set(
    container
      .filter(item =>
        item &&
        typeof item === 'object' &&
        String(item.$id || '').startsWith('event-') &&
        item.event_id
      )
      .map(item => String(item.event_id))
  );

  const crops = container.filter(item =>
    item &&
    typeof item === 'object' &&
    item.$id === 'crop' &&
    unwrapImage(item.image || item)
  );

  crops.sort((left, right) => {
    const leftMatchesEvent = eventIds.has(String(left.ref_event_id || '')) ? 1 : 0;
    const rightMatchesEvent = eventIds.has(String(right.ref_event_id || '')) ? 1 : 0;
    if (leftMatchesEvent !== rightMatchesEvent) {
      return rightMatchesEvent - leftMatchesEvent;
    }
    return Number(right.confidence || 0) - Number(left.confidence || 0);
  });

  return crops[0] || null;
}

function findCropImage(container, attributeValues) {
  const cropNames = [
    'crop_image', 'cropped_image', 'event_crop', 'image_crop',
    'crop', 'snapshot', 'thumbnail'
  ];

  const bestCrop = findBestCrop(container);
  if (bestCrop) {
    const image = unwrapImage(bestCrop.image || bestCrop);
    if (image) return image;
  }

  for (const name of cropNames) {
    const image = unwrapImage(attributeValues[name]);
    if (image) return image;
  }

  if (container && typeof container === 'object' && !Array.isArray(container)) {
    for (const name of cropNames) {
      const image = unwrapImage(container[name]);
      if (image) return image;
    }

    for (const parent of ['event', 'media', 'images', 'result', 'data']) {
      const nested = container[parent];
      if (!nested || typeof nested !== 'object' || Array.isArray(nested)) continue;
      for (const name of cropNames) {
        const image = unwrapImage(nested[name]);
        if (image) return image;
      }
    }
  }

  return '';
}

const values = {};
let metadata = {};

for (const attribute of attributes) {
  if (!attribute || typeof attribute !== 'object') continue;

  if (Object.keys(metadata).length === 0) {
    metadata = {
      instance_id: attribute.instance_id || '',
      tracking_id: attribute.ref_tracking_id || '',
      event_timestamp_ms: Number(attribute.event_timestamp_ms || 0),
      system_timestamp: Number(attribute.system_timestamp || 0),
      event_time: attribute.system_datetime || ''
    };
  }

  const name = String(attribute.name || '');
  if (!name || name === 'face_features') continue;

  let value = attribute.value;
  if (value === 'true') value = true;
  if (value === 'false') value = false;
  values[name] = value;
}

const primaryEvent = attributes.find(item =>
  item &&
  typeof item === 'object' &&
  String(item.$id || '').startsWith('event-')
);
const bestCrop = findBestCrop(payload);
const metadataSource = primaryEvent
  || bestCrop
  || attributes.find(item => item && typeof item === 'object')
  || (payload && typeof payload === 'object' && !Array.isArray(payload) ? payload : {});

metadata = {
  instance_id: metadataSource.instance_id || metadata.instance_id || '',
  tracking_id: metadataSource.ref_tracking_id
    || metadataSource.tracking_id
    || metadata.tracking_id
    || '',
  event_id: metadataSource.event_id
    || metadataSource.ref_event_id
    || bestCrop?.ref_event_id
    || '',
  event_type: primaryEvent?.$id || bestCrop?.$id || metadataSource.$id || '',
  event_timestamp_ms: Number(
    metadataSource.event_timestamp_ms || metadata.event_timestamp_ms || 0
  ),
  system_timestamp: Number(
    metadataSource.system_timestamp || metadata.system_timestamp || 0
  ),
  event_time: metadataSource.system_datetime
    || metadataSource.event_time
    || metadata.event_time
    || ''
};

const cropCount = Array.isArray(payload)
  ? payload.filter(item => item && item.$id === 'crop' && unwrapImage(item.image || item)).length
  : 0;
const cropImage = findCropImage(payload, values);

return {
  ...metadata,
  attribute_count: attributes.filter(
    item => item && item.name && item.name !== 'face_features'
  ).length,
  crop_count: cropCount,
  has_crop: Boolean(cropImage),
  ...(cropImage ? { crop_image: cropImage } : {}),
  ...(bestCrop ? {
    crop_confidence: Number(bestCrop.confidence || 0),
    crop_timestamp_ms: Number(bestCrop.crop_timestamp_ms || 0),
    crop_event_id: bestCrop.ref_event_id || ''
  } : {}),
  smoking: values.smoking ?? false,
  glasses: values.glasses ?? false,
  face_covered: values.face_covered ?? false,
  phone: values.phone ?? false,
  assisted: values.assisted ?? false,
  carrying_bag: values.carrying_bag ?? false,
  tattoo: values.tattoo ?? false,
  age: values.age ?? 'unknown',
  gender: values.gender ?? 'unknown',
  upper_clothing_color: values.upper_clothing_color ?? 'unknown',
  lower_clothing_color: values.lower_clothing_color ?? 'unknown'
};
