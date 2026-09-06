import { describe, expect, it } from 'vitest'
import { defaultPayload, normalizePayload, validatePayload } from '../app/utils/document-form'

const fields = [
  { name: 'title', type: 'text', required: true },
  { name: 'priority', type: 'number', required: false },
  { name: 'status', type: 'select', required: true, is_status: true }
]

describe('defaultPayload', () => {
  it('defaults the status field to the first workflow state', () => {
    const payload = defaultPayload(fields, 'draft')
    expect(payload).toEqual({ title: '', priority: '', status: 'draft' })
  })

  it('leaves non-status fields empty', () => {
    const payload = defaultPayload(fields, '')
    expect(payload).toEqual({ title: '', priority: '', status: '' })
  })
})

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

  it('defaults checkbox fields to false and coerces booleans', () => {
    const checkboxFields = [...fields, { name: 'done', type: 'checkbox', required: false }]
    expect(defaultPayload(checkboxFields, 'draft').done).toBe(false)
    expect(normalizePayload(checkboxFields, { title: 'x', status: 'open', done: 'true' }).done).toBe(true)
    expect(normalizePayload(checkboxFields, { title: 'x', status: 'open', done: true }).done).toBe(true)
  })

  it('strips computed fields from outgoing payloads', () => {
    const computedFields = [...fields, { name: 'summary', type: 'computed', required: false }]
    const payload = normalizePayload(computedFields, { title: 'x', status: 'open', summary: 'stale' })
    expect('summary' in payload).toBe(false)
  })

  it('omits empty optional fields', () => {
    const payload = normalizePayload(fields, { title: 'Fix pump', priority: '', status: 'open' })
    expect(payload).toEqual({ title: 'Fix pump', status: 'open' })
  })

  it('keeps empty required fields so validation can flag them', () => {
    const payload = normalizePayload(fields, { title: '', priority: '', status: 'open' })
    expect(payload.title).toBe('')
  })
})