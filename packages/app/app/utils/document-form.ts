export type Field = { name: string; type: string; required: boolean; is_status?: boolean }

export function defaultPayload(fields: Field[], defaultStatus: string): Record<string, unknown> {
  return Object.fromEntries(fields.map(field => [
    field.name,
    field.is_status ? defaultStatus : field.type === 'checkbox' ? false : ''
  ]))
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
      // Computed fields are derived server-side; never sent.
      if (field.type === 'computed') return []
      const raw = payload[field.name]
      const empty = raw === '' || raw === null || raw === undefined
      // Omit empty values for optional fields so the core does not reject
      // empty strings for select fields.
      if (empty && !field.required) return []
      let value: unknown = raw
      if (field.type === 'number' || field.type === 'currency') {
        value = !empty ? Number(raw) : raw
      } else if (field.type === 'checkbox') {
        value = raw === true || raw === 'true' ? true : raw === false || raw === 'false' ? false : raw
      }
      return [[field.name, value]]
    })
  )
}