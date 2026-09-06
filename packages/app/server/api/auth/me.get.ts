import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const token = getCookie(event, 'lh_session')
  if (!token) throw createError({ statusCode: 401, statusMessage: 'not authenticated' })
  return coreClient(event).me(token)
})