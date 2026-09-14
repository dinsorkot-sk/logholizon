import { coreClient } from '../../../core/client'

export default defineEventHandler(async (event) => {
  const id = getRouterParam(event, 'id')
  const body = await readBody<{ title?: string; due_date?: string; assignee?: string }>(event)
  if (!id?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'id is required' })
  }
  if (!body?.title?.trim()) {
    throw createError({ statusCode: 400, statusMessage: 'title is required' })
  }
  if (body.title.trim().length > 200) {
    throw createError({ statusCode: 400, statusMessage: 'title must be at most 200 characters' })
  }
  if (body.due_date?.trim() && !/^\d{4}-\d{2}-\d{2}$/.test(body.due_date.trim())) {
    throw createError({ statusCode: 400, statusMessage: 'due_date must be YYYY-MM-DD' })
  }
  return coreClient(event).createDocActivity(id, {
    title: body.title.trim(),
    ...(body.due_date?.trim() ? { due_date: body.due_date.trim() } : {}),
    ...(body.assignee?.trim() ? { assignee: body.assignee.trim() } : {})
  })
})
