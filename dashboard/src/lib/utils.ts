import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const formatNum = (num?: number): string => {
  return num !== undefined
    ? num.toLocaleString(undefined, {
        // We want floats to show the precision digits.
        minimumFractionDigits: Number.isSafeInteger(num) ? 0 : 2,
        maximumFractionDigits: 2,
      })
    : "0 (failed to format)"
}
