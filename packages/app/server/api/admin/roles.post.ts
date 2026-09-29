import { coreClient } from '../../core/client'

type CreateRoleBody = { name?: unknown; label?: unknown; description?: unknown }

export default defineEventHandler(async event => {
  const body = await readBody<CreateRoleBody>(event)
  const name = typeof body?.name === 'string' ? body.name.trim() : ''
  const label = typeof body?.label === 'string' ? body.label.trim() : ''
  const description = typeof body?.description === 'string' ? body.description : undefined
  if (!name) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  if (!label) throw createError({ statusCode: 400, statusMessage: 'label is required' })
  return coreClient(event).createRole({ name, label, description })
})
