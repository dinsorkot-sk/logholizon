export type Field = { name: string; type: string; required: boolean; is_status?: boolean }

export function defaultPayload(fields: Field[], defaultStatus: string): Record<string, unknown> {
  return Object.fromEntries(fields.map(field => [field.name, field.is_status ? defaultStatus : '']))
}

export function validatePayload(fields: Field[], payload: Record<string, unknown>): Record<string, string> {
  const errors: Record<string, string> = {}
  for (const field of fields) {
    const value = payload[field.name]
    if (field.required && (value === '' || value === null || value === undefined)) {
      errors[field.name] = `${field.name} is required`
    }
  }
  return errors
}

export function normalizePayload(fields: Field[], payload: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(
    fields.flatMap((field) => {
      const raw = payload[field.name]
      const empty = raw === '' || raw === null || raw === undefined
      // Omit empty values for optional fields so the core does not reject
      // empty strings for select fields.
      if (empty && !field.required) return []
      const value = field.type === 'number' && !empty ? Number(raw) : raw
      return [[field.name, value]]
    })
  )
}