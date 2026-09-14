import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const body = await readBody(event)
  const name = String(body?.name || '').trim()
  const label = String(body?.label || '').trim()
  if (!name || !label) {
    throw createError({ statusCode: 400, statusMessage: 'name and label are required' })
  }
  return coreClient(event).createModule({
    name,
    label,
    description: body?.description,
    icon: body?.icon,
    color: body?.color,
    definition: body?.definition || { entities: [] }
  })
})

