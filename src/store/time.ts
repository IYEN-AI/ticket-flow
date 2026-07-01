export function utcNow(): string {
  return new Date().toISOString().replace(/\.\d{3}Z$/, "Z")
}

export function utcToday(): string {
  return utcNow().slice(0, 10).replaceAll("-", "")
}

export function ageMinutesSince(timestamp: string, now: number): number | undefined {
  const parsed = Date.parse(timestamp)
  if (!Number.isFinite(parsed)) {
    return undefined
  }
  return Math.max(0, Math.floor((now - parsed) / 60_000))
}
