/**
 * Dynamic Icon Map
 *
 * Imports only the icons used by the icon picker, extension registries,
 * and component library — NOT the full lucide-react library.
 *
 * This enables tree-shaking: ~105 icons instead of 1800+ (~80% smaller bundle).
 */

import type { LucideIcon } from 'lucide-react'
import {
  // Common
  Settings, Home, User, Users, Search, Bell, Heart,
  Star, Check, X, Plus, Minus, Filter, Menu,

  // Status
  CheckCircle, XCircle, AlertCircle, AlertTriangle, Info,
  HelpCircle, Circle, Dot, Loader2, Clock, Timer,

  // Arrows
  ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ArrowUpDown,
  ChevronUp, ChevronDown, ChevronLeft, ChevronRight,
  Expand, Shrink, Minimize, Maximize, Move,

  // Media
  Image, Video, Camera, Mic, Volume2, VolumeX, Music,
  Play, Pause, Square, Radio, Tv, Film, Clapperboard,

  // Files
  File, FileText, Folder, FolderOpen, Download, Upload,
  Copy, Clipboard, Scissors, Archive, Trash, Trash2,

  // Devices
  Laptop, Monitor, Smartphone, Tablet, HardDrive, Cpu,
  Wifi, Bluetooth, Usb, Cable, Plug, Power,

  // Charts
  BarChart, BarChart2, BarChart3, BarChart4, LineChart,
  PieChart, TrendingUp, TrendingDown, Activity, Target,
  Flame, Droplet, Wind as WindIcon,

  // Misc
  Sun, Moon, Cloud, CloudRain, Snowflake, CloudLightning,
  MapPin, Navigation, Compass, Globe, Earth,
  Package, Box, ShoppingCart, CreditCard,

  // Extra icons used by registries, dialogs, and backend suggestions
  PackagePlus, FileArchive, BoxSelect, LayoutGrid, LayoutDashboard, Store as StoreIcon,

  // Icons used by backend (suggestions.rs, dashboards.rs, frontend_components.rs)
  Zap, Lightbulb, History, Bot, Send, Map, Webcam, ScanEye,
  Code, Hash, Layers, Sliders, List, Type,
} from 'lucide-react'

/**
 * Map of icon name → component for dynamic lookup by string name.
 * Used by IconPicker, DynamicRegistry, CommunityRegistry, ComponentLibrarySidebar.
 */
export const dynamicIconMap: Record<string, LucideIcon> = {
  // Common
  Settings, Home, User, Users, Search, Bell, Heart,
  Star, Check, X, Plus, Minus, Filter, Menu,

  // Status
  CheckCircle, XCircle, AlertCircle, AlertTriangle, Info,
  HelpCircle, Circle, Dot, Loader2, Clock, Timer,

  // Arrows
  ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ArrowUpDown,
  ChevronUp, ChevronDown, ChevronLeft, ChevronRight,
  Expand, Shrink, Minimize, Maximize, Move,

  // Media
  Image, Video, Camera, Mic, Volume2, VolumeX, Music,
  Play, Pause, Square, Radio, Tv, Film, Clapperboard,

  // Files
  File, FileText, Folder, FolderOpen, Download, Upload,
  Copy, Clipboard, Scissors, Archive, Trash, Trash2,

  // Devices
  Laptop, Monitor, Smartphone, Tablet, HardDrive, Cpu,
  Wifi, Bluetooth, Usb, Cable, Plug, Power,

  // Charts
  BarChart, BarChart2, BarChart3, BarChart4, LineChart,
  PieChart, TrendingUp, TrendingDown, Activity, Target,
  Flame, Droplet, Wind: WindIcon,

  // Misc
  Sun, Moon, Cloud, CloudRain, Snow: Snowflake, Thunder: CloudLightning,
  MapPin, Navigation, Compass, Globe, Earth,
  Package, Box, ShoppingCart, CreditCard,

  // Extra
  PackagePlus, FileArchive, BoxSelect, LayoutGrid, LayoutDashboard, Store: StoreIcon,

  // Backend-used icons
  Zap, Lightbulb, History, Bot, Send, Map, Webcam, ScanEye,
  Code, Hash, Layers, Sliders, List, Type,
}

/**
 * Get an icon component by name, with optional fallback.
 * Returns null if not found and no fallback provided.
 */
export function getDynamicIcon(name: string, fallback?: LucideIcon): LucideIcon | null {
  return dynamicIconMap[name] ?? fallback ?? null
}
