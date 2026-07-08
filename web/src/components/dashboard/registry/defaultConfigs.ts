/**
 * Default component configurations for dashboard components.
 * Used when adding a new component to the grid.
 */

/** Empty placeholder — image components show default empty state when no data source is bound */
export const IMAGE_PLACEHOLDER_SRC = ''

export const DEFAULT_COMPONENT_CONFIGS: Record<string, Record<string, unknown>> = {
  // Charts (series color set dynamically via chartColorsHex[0])
  'line-chart': {
    series: [{ name: 'Value', data: [10, 25, 15, 30, 28, 35, 20] }],
    labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
  },
  'area-chart': {
    series: [{ name: 'Value', data: [10, 25, 15, 30, 28, 35, 20] }],
    labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
  },
  'bar-chart': {
    data: [{ name: 'A', value: 30 }, { name: 'B', value: 50 }, { name: 'C', value: 20 }],
  },
  'pie-chart': {
    data: [{ name: 'A', value: 30 }, { name: 'B', value: 50 }, { name: 'C', value: 20 }],
  },
  // Indicators
  'sparkline': {
    data: [12, 19, 15, 25, 22, 30, 28],
  },
  'progress-bar': {
    value: 65,
    min: 0,
    max: 100,
  },
  'led-indicator': {
    rules: [],
  },
  // Controls
  'toggle-switch': {
    size: 'md',
  },
  // Display & Content
  'image-display': {
    src: IMAGE_PLACEHOLDER_SRC,
    alt: 'Sample Image',
    fit: 'contain',
    rounded: true,
    zoomable: true,
  },
  'image-history': {
    dataSource: undefined,
    fit: 'fill',
    rounded: true,
    limit: 50,
    timeRange: 24,
  },
  'web-display': {
    src: 'https://example.com',
    title: 'Website',
    sandbox: true,
    showHeader: true,
  },
  'markdown-display': {
    content: '# Title\n\nThis is **markdown** content.\n\n- Item 1\n- Item 2\n\n`code example`',
    variant: 'default',
  },
  'video-display': {
    src: '',
    type: 'file',
    autoplay: false,
    muted: true,
    controls: true,
    loop: false,
    fit: 'contain',
    rounded: true,
    showFullscreen: true,
  },
  'map-display': {
    center: { lat: 39.9042, lng: 116.4074 },
    zoom: 10,
    minZoom: 2,
    maxZoom: 18,
    showControls: true,
    showLayers: true,
    showFullscreen: true,
    interactive: true,
    tileLayer: 'osm',
    markers: [
      { id: '1', latitude: 39.9042, longitude: 116.4074, label: 'Beijing', status: 'online' },
      { id: '2', latitude: 31.2304, longitude: 121.4737, label: 'Shanghai', status: 'online' },
      { id: '3', latitude: 23.1291, longitude: 113.2644, label: 'Guangzhou', status: 'warning' },
    ],
  },
  'custom-layer': {
    backgroundType: 'grid',
    gridSize: 20,
    showControls: true,
    showFullscreen: true,
    interactive: true,
    editable: false,
  },
  // Business Components
  'agent-monitor-widget': {},
  'ai-analyst': {},
}
