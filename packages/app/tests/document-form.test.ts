import { describe, expect, it } from 'vitest'
import { normalizePayload, validatePayload } from '../app/utils/document-form'

const fields = [
  { name: 'title', type: 'text', required: true },
  { name: 'priority', type: 'number', required: false },
  { name: 'status', type: 'select', required: true }
]

describe('validatePayload', () => {
  it('flags missing required fields', () => {
    const errors = validatePayload(fields, { title: '', status: 'open' })
    expect(errors.title).toBe('title is required')
    expect(errors.status).toBeUndefined()
  })

  it('accepts complete payload', () => {
    expect(validatePayload(fields, { title: 'Fix pump', status: 'open' })).toEqual({})
  })
})

describe('normalizePayload', () => {
  it('converts number strings to numbers', () => {
    const payload = normalizePayload(fields, { title: 'Fix pump', priority: '3', status: 'open' })
    expect(payload.priority).toBe(3)
  })

  it('keeps empty number as empty string', () => {
    const payload = normalizePayload(fields, { title: 'Fix pump', priority: '', status: 'open' })
    expect(payload.priority).toBe('')
  })
})