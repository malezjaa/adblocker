import {type ClassValue, clsx} from "clsx"
import {twMerge} from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export const chartColors = {
  total: "text-foreground",
  blocked: "text-muted-foreground",
}

export function countryFlag(countryCode: string) {
  const code = countryCode.trim().toUpperCase()
  if (code.length !== 2) return "🏳️"
  return String.fromCodePoint(
    ...Array.from(code).map((char) => 127397 + char.charCodeAt(0))
  )
}