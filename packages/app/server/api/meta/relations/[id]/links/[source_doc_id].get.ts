export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig(event)
  const id = getRouterParam(event, 'id')
  const sourceDocId = getRouterParam(event, 'source_doc_id')
  return await $fetch(`${config.coreApiBase}/v1/meta/relations/${encodeURIComponent(id!)}/links/${encodeURIComponent(sourceDocId!)}`, {
    headers: { authorization: getHeader(event, 'authorization') || '' }
  })
})
