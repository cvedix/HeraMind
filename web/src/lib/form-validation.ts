/**
 * Form validation utilities for inline error messages.
 * Returns null for valid values, error message string for invalid ones.
 */

export function validateRequired(value: string, field: string): string | null {
  return value.trim() ? null : `${field} is required`;
}

export function validateLength(
  value: string,
  field: string,
  min: number,
  max: number
): string | null {
  const len = value.trim().length;
  if (len < min) return `${field} must be at least ${min} characters`;
  if (len > max) return `${field} must be at most ${max} characters`;
  return null;
}

export function validatePattern(
  value: string,
  field: string,
  pattern: RegExp,
  message: string
): string | null {
  return pattern.test(value) ? null : message;
}

export function validateIdentifier(value: string, field: string): string | null {
  if (!value.trim()) return `${field} is required`;
  if (!/^[a-zA-Z0-9_-]+$/.test(value.trim())) {
    return `${field} can only contain letters, numbers, hyphens, and underscores`;
  }
  return null;
}

export function validateUrl(value: string, field: string): string | null {
  if (!value.trim()) return `${field} is required`;
  try {
    new URL(value);
    return null;
  } catch {
    return `${field} must be a valid URL`;
  }
}

export function validatePort(value: number, field: string): string | null {
  if (!Number.isInteger(value) || value < 1 || value > 65535) {
    return `${field} must be between 1 and 65535`;
  }
  return null;
}

/**
 * Combines multiple validation results, returning the first error found.
 */
export function composeValidators(
  ...validators: (string | null)[]
): string | null {
  return validators.find((v) => v !== null) ?? null;
}
