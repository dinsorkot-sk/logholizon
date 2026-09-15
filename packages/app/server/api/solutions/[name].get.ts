import { solutionPackage } from '#shared/utils/solution-catalog'

export default defineEventHandler((event) => {
  const name = getRouterParam(event, 'name') || ''
  const pkg = solutionPackage(name)
  if (!pkg) {
    throw createError({ statusCode: 404, statusMessage: `unknown solution: ${name}` })
  }
  return pkg
})
