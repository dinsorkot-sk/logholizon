import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const token = getCookie(event, 'lh_session')
  if (token) {
    try {
      await coreClient(event).logout(token)
    } catch {
      // ignore: token may already be invalid
    }
  }
  deleteCookie(event, 'lh_session', { path: '/' })
  return { success: true }
})