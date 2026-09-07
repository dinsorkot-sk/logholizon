import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  const bytes = await coreClient(event).downloadDocAttachment(id)
  return new Response(new Uint8Array(bytes), {
    headers: {
      'content-type': 'application/octet-stream',
      'content-disposition': `attachment; filename="${id}"`
    }
  })
})
