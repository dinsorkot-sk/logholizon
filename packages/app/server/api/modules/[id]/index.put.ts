import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id') || ''
  const body = await readBody(event)
  const definition = body?.definition ?? body
  return coreClient(event).updateModule(id, {
    label: body?.label,
    description: body?.description,
    icon: body?.icon,
    color: body?.color,
    definition
  })
})
