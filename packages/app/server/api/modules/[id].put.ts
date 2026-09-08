import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  return coreClient(event).updateModule(id, {
    label: body?.label,
    description: body?.description,
    icon: body?.icon,
    color: body?.color,
    definition: body?.definition
  })
})
