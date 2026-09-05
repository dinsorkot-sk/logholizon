import { coreClient } from '../../../core/client'

type UpdateOptionBody = { value: string; label: string }

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<UpdateOptionBody>(event)
  if (!id?.trim()) throw createError({ statusCode: 400, statusMessage: 'id is required' })
  if (!body?.value?.trim() || !body?.label?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'value and label are required' })
  }
  return coreClient().updateFieldOption(id, { value: body.value, label: body.label })
})