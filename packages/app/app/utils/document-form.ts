export type Field = { name: string; type: string; required: boolean }

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
    fields.map((field) => {
      const raw = payload[field.name]
      const value = field.type === 'number' && raw !== '' && raw !== null && raw !== undefined
        ? Number(raw)
        : raw
      return [field.name, value]
    })
  )
}