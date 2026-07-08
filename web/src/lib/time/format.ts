/**
 * Time formatting utilities for HeraMind
 *
 * This module provides centralized time formatting functions that respect:
 * - User's display time format preference (12h/24h)
 * - User's display timezone preference (local/utc)
 * - Global system timezone (IANA format like "Asia/Shanghai")
 */

// ============================================================================
// Types
// ============================================================================

export type TimeFormat = "12h" | "24h"
export type DisplayTimeZone = "local" | "utc"

export interface TimePreferences {
  timeFormat: TimeFormat
  timeZone: DisplayTimeZone
}

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_PREFERENCES: TimePreferences = {
  timeFormat: "24h",
  timeZone: "local",
}

const PREFERENCES_KEY = "heramind_preferences"

// ============================================================================
// Preferences Management
// ============================================================================

/**
 * Load user's time display preferences from localStorage
 */
export function loadTimePreferences(): TimePreferences {
  try {
    const saved = localStorage.getItem(PREFERENCES_KEY)
    if (saved) {
      const parsed = JSON.parse(saved)
      return {
        timeFormat: parsed.timeFormat || DEFAULT_PREFERENCES.timeFormat,
        timeZone: parsed.timeZone || DEFAULT_PREFERENCES.timeZone,
      }
    }
  } catch (e) {
    console.error("Failed to load time preferences:", e)
  }
  return DEFAULT_PREFERENCES
}

// ============================================================================
// Time Formatting Functions
// ============================================================================

/**
 * Format time based on user preferences
 * @param date - Date to format
 * @param preferences - User preferences (optional, loads from storage if not provided)
 * @returns Formatted time string
 */
export function formatTime(
  date: Date,
  preferences?: TimePreferences
): string {
  const prefs = preferences || loadTimePreferences()
  let displayDate = date

  // Convert to UTC if needed
  if (prefs.timeZone === "utc") {
    displayDate = new Date(date.toUTCString())
  }

  const hours = displayDate.getHours()
  const minutes = displayDate.getMinutes().toString().padStart(2, "0")
  const seconds = displayDate.getSeconds().toString().padStart(2, "0")

  if (prefs.timeFormat === "12h") {
    const period = hours >= 12 ? "PM" : "AM"
    const displayHours = hours % 12 || 12
    return `${displayHours}:${minutes}:${seconds} ${period}`
  }

  return `${hours.toString().padStart(2, "0")}:${minutes}:${seconds}`
}

/**
 * Format time without seconds
 * @param date - Date to format
 * @param preferences - User preferences (optional)
 * @returns Formatted time string (HH:MM or HH:MM AM/PM)
 */
export function formatTimeShort(
  date: Date,
  preferences?: TimePreferences
): string {
  const formatted = formatTime(date, preferences)
  // Remove seconds part
  if (formatted.includes(" ")) {
    // 12h format: "HH:MM:SS PM" -> "HH:MM PM"
    const parts = formatted.split(" ")
    const timePart = parts[0].substring(0, 5) // HH:MM
    return `${timePart} ${parts[1]}`
  }
  // 24h format: "HH:MM:SS" -> "HH:MM"
  return formatted.substring(0, 5)
}

/**
 * Format date and time based on user preferences
 * @param date - Date to format
 * @param preferences - User preferences (optional)
 * @returns Formatted date time string
 */
export function formatDateTime(
  date: Date,
  preferences?: TimePreferences
): string {
  const prefs = preferences || loadTimePreferences()
  const year = date.getFullYear()
  const month = (date.getMonth() + 1).toString().padStart(2, "0")
  const day = date.getDate().toString().padStart(2, "0")

  if (prefs.timeZone === "utc") {
    // For UTC, add UTC marker
    return `${year}-${month}-${day} ${formatTime(date, prefs)} UTC`
  }

  return `${year}-${month}-${day} ${formatTime(date, prefs)}`
}

/**
 * Format date only (YYYY-MM-DD)
 * @param date - Date to format
 * @returns Formatted date string
 */
export function formatDate(date: Date): string {
  const year = date.getFullYear()
  const month = (date.getMonth() + 1).toString().padStart(2, "0")
  const day = date.getDate().toString().padStart(2, "0")
  return `${year}-${month}-${day}`
}

/**
 * Format time with timezone information
 * @param date - Date to format
 * @param timezone - IANA timezone string (e.g., "Asia/Shanghai")
 * @param preferences - User preferences (optional)
 * @returns Formatted time string with timezone
 */
export function formatTimeWithTimezone(
  date: Date,
  timezone: string,
  preferences?: TimePreferences
): string {
  const prefs = preferences || loadTimePreferences()

  try {
    // Use Intl.DateTimeFormat for timezone conversion
    const formatter = new Intl.DateTimeFormat("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: prefs.timeFormat === "12h",
      timeZone: timezone,
    })

    return formatter.format(date)
  } catch {
    // Fallback if timezone is invalid
    return formatTime(date, prefs)
  }
}

/**
 * Format current time in a specific timezone
 * @param timezone - IANA timezone string (e.g., "Asia/Shanghai")
 * @param preferences - User preferences (optional)
 * @returns Formatted current time string with timezone name
 */
export function formatCurrentTimeInTimezone(
  timezone: string,
  preferences?: TimePreferences
): string {
  const prefs = preferences || loadTimePreferences()
  const now = new Date()

  try {
    // Format the time in the specified timezone
    const formatter = new Intl.DateTimeFormat("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hour12: prefs.timeFormat === "12h",
      timeZone: timezone,
      timeZoneName: "short",
    })

    return formatter.format(now)
  } catch {
    // Fallback if timezone is invalid
    return formatTime(now, prefs)
  }
}

/**
 * Get current time information object
 * @param timezone - IANA timezone string (optional)
 * @returns Time information object
 */
export function getCurrentTimeInfo(timezone?: string): {
  utc: string
  local: string
  timezone?: string
  timestamp: number
} {
  const now = new Date()

  const info: {
    utc: string
    local: string
    timestamp: number
    timezone?: string
  } = {
    utc: now.toUTCString(),
    local: now.toString(),
    timestamp: now.getTime(),
  }

  if (timezone) {
    try {
      const formatter = new Intl.DateTimeFormat("en-US", {
        timeZone: timezone,
        dateStyle: "long",
        timeStyle: "long",
      })
      info.timezone = formatter.format(now)
    } catch {
      info.timezone = now.toString()
    }
  }

  return info
}

/**
 * Format a timestamp as relative time (e.g., "2 hours ago")
 * @param timestamp - Unix timestamp in milliseconds
 * @returns Relative time string
 */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp

  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (seconds < 60) {
    return "just now"
  } else if (minutes < 60) {
    return `${minutes}m ago`
  } else if (hours < 24) {
    return `${hours}h ago`
  } else if (days < 7) {
    return `${days}d ago`
  } else {
    // For older timestamps, return the date
    return formatDate(new Date(timestamp))
  }
}

/**
 * Parse an ISO 8601 date string to Date
 * @param isoString - ISO 8601 date string
 * @returns Date object
 */
export function parseISODate(isoString: string): Date {
  return new Date(isoString)
}

/**
 * Convert a Unix timestamp (seconds) to Date
 * @param timestamp - Unix timestamp in seconds
 * @returns Date object
 */
export function fromUnixTimestamp(timestamp: number): Date {
  return new Date(timestamp * 1000)
}

/**
 * Convert a Date to Unix timestamp (seconds)
 * @param date - Date object
 * @returns Unix timestamp in seconds
 */
export function toUnixTimestamp(date: Date): number {
  return Math.floor(date.getTime() / 1000)
}

// ============================================================================
// Timezone Utilities
// ============================================================================

/**
 * Common timezone options for selection (IDs only)
 */
export const COMMON_TIMEZONE_IDS = [
  "Asia/Shanghai",
  "Asia/Ho_Chi_Minh",
  "Asia/Tokyo",
  "Asia/Seoul",
  "Asia/Singapore",
  "Asia/Hong_Kong",
  "Asia/Dubai",
  "Europe/London",
  "Europe/Paris",
  "Europe/Berlin",
  "Europe/Moscow",
  "America/New_York",
  "America/Los_Angeles",
  "America/Chicago",
  "America/Toronto",
  "America/Sao_Paulo",
  "Australia/Sydney",
  "Pacific/Auckland",
  "UTC",
] as const

export type CommonTimezoneId = (typeof COMMON_TIMEZONE_IDS)[number]

/**
 * Timezone info structure
 */
export interface TimezoneInfo {
  id: string
  name: string
}

/**
 * Get timezone display name from i18n
 * This function should be called with a translation function
 * @param timezoneId - IANA timezone string
 * @param t - Translation function from useTranslation
 * @returns Localized timezone name
 */
export function getTimezoneDisplayName(
  timezoneId: string,
  t: (key: string) => string
): string {
  const key = `settings:timezones.${timezoneId.replace(/\//g, ".")}`
  const translated = t(key)
  // If translation not found, return the ID itself
  if (translated === key || translated === `settings:timezones.${timezoneId}`) {
    // Fallback to city name from ID
    const city = timezoneId.split("/").pop() || timezoneId
    return city.replace(/_/g, " ")
  }
  return translated
}

/**
 * Get localized timezone list
 * @param t - Translation function from useTranslation
 * @returns Array of timezone info with localized names
 */
export function getLocalizedTimezones(t: (key: string) => string): TimezoneInfo[] {
  return COMMON_TIMEZONE_IDS.map((id) => ({
    id,
    name: getTimezoneDisplayName(id, t),
  }))
}

/**
 * Find a timezone by its ID or return the default
 * @param timezoneId - IANA timezone string
 * @param t - Optional translation function for localized name
 * @returns Timezone object or default
 */
export function findTimezone(
  timezoneId: string,
  t?: (key: string) => string
): {
  id: string
  name: string
} {
  const exists = COMMON_TIMEZONE_IDS.includes(timezoneId as CommonTimezoneId)
  if (exists) {
    return {
      id: timezoneId,
      name: t ? getTimezoneDisplayName(timezoneId, t) : timezoneId,
    }
  }
  return {
    id: "Asia/Shanghai",
    name: t ? getTimezoneDisplayName("Asia/Shanghai", t) : "Asia/Shanghai",
  }
}

/**
 * Validate if a timezone string is a valid IANA timezone
 * @param timezone - Timezone string to validate
 * @returns true if valid
 */
export function isValidTimezone(timezone: string): boolean {
  try {
    new Intl.DateTimeFormat("en-US", { timeZone: timezone })
    return true
  } catch {
    return false
  }
}

/**
 * Get the user's browser timezone
 * @returns IANA timezone string
 */
export function getBrowserTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone
}
