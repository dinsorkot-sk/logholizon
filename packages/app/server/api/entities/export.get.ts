import { coreClient } from '../../core/client'

export default defineEventHandler(async (event) => {
  const bytes = await coreClient(event).exportWorkbook()
  return new Response(bytes, {
    headers: {
      'content-type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      'content-disposition': 'attachment; filename="logholizon.xlsx"'
    }
  })
})
