import { coreClient } from '../../core/client'

type RestoreBody = { path: string; force?: boolean }

export default defineEventHandler(async (event) => {
  const body = await readBody<RestoreBody>(event)
  if (!body?.path?.trim()) throw createError({ statusCode: 400, statusMessage: 'path is required' })
  if (!body.force) throw createError({ statusCode: 400, statusMessage: 'force is required' })
  return coreClient(event).restoreBackup(body.path)
})