import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const name = getRouterParam(event, 'name')
  if (!name?.trim()) throw createError({ statusCode: 400, statusMessage: 'name is required' })
  const blob = await coreClient().downloadBackup(name)
  const bytes = new Uint8Array(await blob.arrayBuffer())
  return new Response(bytes, {
    headers: {
      'content-type': 'application/octet-stream',
      'content-disposition': `attachment; filename="${name}"`
    }
  })
})