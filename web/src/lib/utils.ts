import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Re-export design system utilities used by consumers via @/lib/utils
export {
  formatValue,
  toNumberArray,
  clamp,
  normalize,
} from '@/design-system/utils/format'

export { getIconForEntity } from '@/design-system/icons'
export { EntityIcon } from '@/design-system/icons'
