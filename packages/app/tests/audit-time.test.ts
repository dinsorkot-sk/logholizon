import { describe, expect, it, vi } from 'vitest'
import { absoluteTime, actionLabel, parseDate, relativeTime } from '../app/utils/audit-time'

describe('parseDate', () => {
  it('treats SQLite timestamps without timezone as UTC', () => {
    const date = parseDate('2026-09-06 12:00:00')
    expect(date.toISOString()).toBe('2026-09-06T12:00:00.000Z')
  })

  it('keeps explicit timezone suffixes as-is', () => {
    const date = parseDate('2026-09-06T12:00:00+07:00')
    expect(date.toISOString()).toBe('2026-09-06T05:00:00.000Z')
  })
})

describe('relativeTime', () => {
  it('returns just now for recent timestamps', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-09-06T12:00:30.000Z'))
    try {
      expect(relativeTime('2026-09-06 12:00:00')).toBe('just now')
    } finally {
      vi.useRealTimers()
    }
  })

  it('formats minutes, hours, and days', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-09-06T12:00:00.000Z'))
    try {
      expect(relativeTime('2026-09-06 11:30:00')).toBe('30 min ago')
      expect(relativeTime('2026-09-06 09:00:00')).toBe('3 hours ago')
      expect(relativeTime('2026-09-04 12:00:00')).toBe('2 days ago')
    } finally {
      vi.useRealTimers()
    }
  })

  it('returns the raw value for invalid input', () => {
    expect(relativeTime('not-a-date')).toBe('not-a-date')
  })
})

describe('absoluteTime', () => {
  it('returns the raw value for invalid input', () => {
    expect(absoluteTime('not-a-date')).toBe('not-a-date')
  })
})

describe('actionLabel', () => {
  it('maps known actions to labels', () => {
    expect(actionLabel('create')).toBe('Created')
    expect(actionLabel('transition')).toBe('Status changed')
  })

  it('falls back to the raw action for unknown actions', () => {
    expect(actionLabel('custom_action')).toBe('custom_action')
  })
})
